use bevy::{
    core_pipeline::bloom::BloomSettings,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_xpbd_3d::prelude::*;

use crate::{
    npc::{Health, Npc},
    physics::Layer,
    player::Player,
    vfx::DamageParticlesEvent,
};

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Laser>()
            .init_resource::<Weapons>()
            .add_systems(Startup, setup_weapons)
            .add_systems(
                Update,
                (
                    (laser_target_npc, laser_target_player),
                    laser_shoot_ray,
                    laser_ray_update,
                    laser_ray_despawn,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
pub struct Weapons {
    pub laser_mesh: Handle<Mesh>,
    pub player_laser_material: Handle<StandardMaterial>,
    pub npc_laser_material: Handle<StandardMaterial>,
}

fn setup_weapons(
    mut weapons: ResMut<Weapons>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    weapons.laser_mesh = meshes.add(
        Mesh::try_from(shape::Cylinder {
            radius: 0.05,
            height: 1.,
            resolution: 8,
            segments: 1,
        })
        .unwrap(),
    );
    weapons.player_laser_material = materials.add(StandardMaterial {
        base_color: Color::YELLOW,
        emissive: Color::YELLOW,
        perceptual_roughness: 1.0,
        metallic: 0.,
        reflectance: 0.,
        ..default()
    });
    weapons.npc_laser_material = materials.add(StandardMaterial {
        base_color: Color::ORANGE_RED,
        emissive: Color::ORANGE_RED,
        perceptual_roughness: 1.0,
        metallic: 0.,
        reflectance: 0.,
        ..default()
    });
}

#[derive(Component, Default, Reflect)]
pub struct Laser {
    pub range: f32,
    pub dps: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub target: Option<Entity>,
    pub ray: Option<Entity>,
    pub time_ended: f32,
    pub is_player: bool,
}

impl Laser {
    pub fn new(range: f32, dps: f32, duration: f32, cooldown: f32, is_player: bool) -> Self {
        Self {
            range,
            dps,
            duration,
            cooldown,
            target: None,
            ray: None,
            time_ended: 0.,
            is_player,
        }
    }
}

fn laser_target_npc(
    time: Res<Time>,
    q_space: SpatialQuery,
    mut q_laser: Query<(&mut Laser, &Transform), With<Player>>,
    q_npc: Query<&Transform, With<Npc>>,
) {
    for (mut laser, tr_player) in &mut q_laser {
        if laser.target.is_some() || time.elapsed_seconds() - laser.time_ended < laser.cooldown {
            continue;
        }
        let pos = tr_player.translation;
        if let Some((hit_ent, _)) = q_space
            .shape_intersections(
                &Collider::ball(laser.range),
                pos,
                Quat::default(),
                SpatialQueryFilter::new().with_masks([Layer::NPC]),
            )
            .iter()
            .filter_map(|ent| q_npc.get(*ent).ok().map(|tr| (ent, tr.translation)))
            .min_by(|(_, pos1), (_, pos2)| {
                (*pos1 - pos)
                    .length()
                    .partial_cmp(&(*pos2 - pos).length())
                    .unwrap()
            })
        {
            laser.target = Some(*hit_ent);
        }
    }
}

fn laser_target_player(
    time: Res<Time>,
    mut q_laser: Query<(&mut Laser, &Transform), Without<Player>>,
    q_player: Query<(Entity, &Transform), With<Player>>,
) {
    for (mut laser, tr_src) in &mut q_laser {
        if laser.target.is_some() || time.elapsed_seconds() - laser.time_ended < laser.cooldown {
            continue;
        }
        let src_pos = tr_src.translation;
        let Some((player_ent, player_pos)) = q_player
            .iter()
            .map(|(ent, tr)| (ent, tr.translation))
            .min_by(|p1, p2| {
                (src_pos - p1.1)
                    .length()
                    .partial_cmp(&(src_pos - p2.1).length())
                    .unwrap()
            })
        else {
            continue;
        };
        if laser.range > (player_pos - src_pos).length() {
            laser.target = Some(player_ent);
        }
    }
}

#[derive(Component)]
pub struct LaserRay {
    pub source: Entity,
    pub target: Entity,
    pub time_started: f32,
    pub dead: bool,
    pub vfx_started: bool,
}

#[derive(Component)]
pub struct LaserRayMesh;

fn laser_shoot_ray(
    time: Res<Time>,
    weapons: Res<Weapons>,
    mut q_laser: Query<(Entity, &mut Laser)>,
    mut cmd: Commands,
) {
    for (source, mut laser) in &mut q_laser {
        if laser.ray.is_none() {
            let Some(target) = laser.target else {
                continue;
            };
            let id = cmd
                .spawn((
                    LaserRay {
                        source,
                        target,
                        time_started: time.elapsed_seconds(),
                        dead: false,
                        vfx_started: false,
                    },
                    SpatialBundle::from_transform(Transform::from_xyz(0., -10., 0.)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        LaserRayMesh,
                        PbrBundle {
                            transform: Transform::IDENTITY,
                            mesh: weapons.laser_mesh.clone(),
                            material: if laser.is_player {
                                weapons.player_laser_material.clone()
                            } else {
                                weapons.npc_laser_material.clone()
                            },
                            ..default()
                        },
                        BloomSettings {
                            intensity: 0.9,
                            ..default()
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                })
                .id();
            laser.ray = Some(id);
        }
    }
}

fn laser_ray_update(
    time: Res<Time>,
    mut ev_damage_particles: EventWriter<DamageParticlesEvent>,
    mut q_ray: Query<(&mut LaserRay, &mut Transform, &Children), Without<LaserRayMesh>>,
    mut q_ray_mesh: Query<&mut Transform, (With<LaserRayMesh>, Without<Laser>)>,
    mut q_targets: Query<
        (&Transform, Option<&Laser>, Option<&mut Health>, Has<Player>),
        (Without<LaserRay>, Without<LaserRayMesh>),
    >,
) {
    for (mut ray, mut tr_ray, children) in &mut q_ray {
        if ray.dead {
            continue;
        };
        let (s, dps, duration, color) = {
            let Ok((tr_laser, Some(laser), _, is_player)) = q_targets.get(ray.source) else {
                ray.dead = true;
                continue;
            };
            (
                tr_laser.translation + if is_player { Vec3::Y * 0.8 } else { Vec3::ZERO },
                laser.dps,
                laser.duration,
                if is_player {
                    Color::YELLOW
                } else {
                    Color::ORANGE_RED
                },
            )
        };
        let Ok((tr_target, _, Some(mut health), _)) = q_targets.get_mut(ray.target) else {
            ray.dead = true;
            continue;
        };
        let Some(child) = children.first() else {
            ray.dead = true;
            continue;
        };
        let Ok(mut tr_ray_mesh) = q_ray_mesh.get_mut(*child) else {
            ray.dead = true;
            continue;
        };
        if time.elapsed_seconds() - ray.time_started > duration {
            ray.dead = true;
            continue;
        }

        let t = tr_target.translation;
        let ts = t - s;
        let dir = ts.normalize();
        tr_ray.translation = (s + t) / 2.;
        tr_ray.look_to(dir.any_orthonormal_vector(), dir);
        tr_ray_mesh.scale = Vec3::new(1., ts.length(), 1.);

        if !ray.vfx_started {
            ray.vfx_started = true;
            ev_damage_particles.send(DamageParticlesEvent {
                position: t - dir * 0.5,
                normal: -dir,
                color,
            });
        }

        health.take_damage(time.delta_seconds() * dps);
    }
}

fn laser_ray_despawn(
    time: Res<Time>,
    q_ray: Query<(Entity, &LaserRay)>,
    mut q_laser: Query<&mut Laser>,
    mut cmd: Commands,
) {
    for (ray_ent, ray) in &q_ray {
        if ray.dead {
            cmd.entity(ray_ent).despawn_recursive();
            if let Ok(mut laser) = q_laser.get_mut(ray.source) {
                laser.ray = None;
                laser.target = None;
                laser.time_ended = time.elapsed_seconds();
            }
        }
    }
}
