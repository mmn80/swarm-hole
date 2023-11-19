use bevy::{
    core_pipeline::bloom::BloomSettings,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_xpbd_3d::prelude::*;
use serde::Deserialize;

use crate::{app::AppState, npc::Npc, physics::Layer, player::Player, vfx::DamageParticlesEvent};

use super::health::TakeDamageEvent;

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<LaserState>()
            .init_resource::<LaserHandles>()
            .add_systems(Startup, setup_assets)
            .add_systems(
                Update,
                (
                    init_laser_state,
                    (
                        (laser_target_npc, laser_target_player),
                        laser_shoot_ray,
                        laser_ray_update,
                        laser_ray_despawn,
                    )
                        .chain(),
                )
                    .run_if(in_state(AppState::Run)),
            )
            .add_systems(OnEnter(AppState::Cleanup), cleanup_laser_rays);
    }
}

#[derive(Resource, Default)]
pub struct LaserHandles {
    pub laser_mesh: Handle<Mesh>,
    pub player_laser_material: Handle<StandardMaterial>,
    pub npc_laser_material: Handle<StandardMaterial>,
}

const PLAYER_LASER_COLOR: Color = Color::rgb(5.0, 5.0, 0.0);
const NPC_LASER_COLOR: Color = Color::rgb(5.0, 2.0, 0.0);

fn setup_assets(
    mut handles: ResMut<LaserHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    handles.laser_mesh = meshes.add(
        Mesh::try_from(shape::Cylinder {
            radius: 0.05,
            height: 1.,
            resolution: 8,
            segments: 1,
        })
        .unwrap(),
    );
    handles.player_laser_material = materials.add(StandardMaterial {
        base_color: PLAYER_LASER_COLOR,
        emissive: PLAYER_LASER_COLOR,
        perceptual_roughness: 1.0,
        metallic: 0.,
        reflectance: 0.,
        ..default()
    });
    handles.npc_laser_material = materials.add(StandardMaterial {
        base_color: NPC_LASER_COLOR,
        emissive: NPC_LASER_COLOR,
        perceptual_roughness: 1.0,
        metallic: 0.,
        reflectance: 0.,
        ..default()
    });
}

#[derive(Copy, Clone, Reflect, Component, Debug, Deserialize)]
pub struct Laser {
    pub range: f32,
    pub dps: f32,
    pub duration: f32,
    pub cooldown: f32,
}

fn init_laser_state(q_laser: Query<Entity, (With<Laser>, Without<LaserState>)>, mut cmd: Commands) {
    for ent in &q_laser {
        cmd.entity(ent).insert(LaserState {
            target: None,
            ray: None,
            time_ended: 0.,
        });
    }
}

#[derive(Component, Reflect)]
pub struct LaserState {
    pub target: Option<Entity>,
    pub ray: Option<Entity>,
    pub time_ended: f32,
}

fn laser_target_npc(
    time: Res<Time>,
    q_space: SpatialQuery,
    mut q_laser: Query<(&mut LaserState, &Laser, &Transform), With<Player>>,
    q_npc: Query<&Transform, With<Npc>>,
) {
    for (mut laser, laser_config, tr_player) in &mut q_laser {
        if laser.target.is_some()
            || time.elapsed_seconds() - laser.time_ended < laser_config.cooldown
        {
            continue;
        }
        let pos = tr_player.translation;
        if let Some((hit_ent, _)) = q_space
            .shape_intersections(
                &Collider::ball(laser_config.range),
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
    mut q_laser: Query<(&mut LaserState, &Laser, &Transform), Without<Player>>,
    q_player: Query<(Entity, &Transform), With<Player>>,
) {
    for (mut laser, laser_config, tr_src) in &mut q_laser {
        if laser.target.is_some()
            || time.elapsed_seconds() - laser.time_ended < laser_config.cooldown
        {
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
        if laser_config.range > (player_pos - src_pos).length() {
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
    weapons: Res<LaserHandles>,
    mut q_laser: Query<(Entity, &mut LaserState, Has<Player>)>,
    mut cmd: Commands,
) {
    for (source, mut laser, is_player) in &mut q_laser {
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
                    SpatialBundle::default(),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        LaserRayMesh,
                        PbrBundle {
                            transform: Transform::IDENTITY,
                            mesh: weapons.laser_mesh.clone(),
                            material: if is_player {
                                weapons.player_laser_material.clone()
                            } else {
                                weapons.npc_laser_material.clone()
                            },
                            visibility: Visibility::Hidden,
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
    mut q_ray_mesh: Query<
        (&mut Transform, &mut Visibility),
        (With<LaserRayMesh>, Without<LaserState>),
    >,
    mut q_targets: Query<
        (&Transform, Option<&Laser>, Has<Player>),
        (Without<LaserRay>, Without<LaserRayMesh>),
    >,
    mut ev_take_damage: EventWriter<TakeDamageEvent>,
) {
    for (mut ray, mut tr_ray, children) in &mut q_ray {
        if ray.dead {
            continue;
        };
        let (s, dps, duration, color) = {
            let Ok((tr_laser, Some(laser_config), is_player)) = q_targets.get(ray.source) else {
                ray.dead = true;
                continue;
            };
            (
                tr_laser.translation + if is_player { Vec3::Y * 0.8 } else { Vec3::ZERO },
                laser_config.dps,
                laser_config.duration,
                if is_player {
                    PLAYER_LASER_COLOR
                } else {
                    NPC_LASER_COLOR
                },
            )
        };
        let Ok((tr_target, _, _)) = q_targets.get_mut(ray.target) else {
            ray.dead = true;
            continue;
        };
        let Some(child) = children.first() else {
            ray.dead = true;
            continue;
        };
        let Ok((mut tr_ray_mesh, mut vis_ray_mesh)) = q_ray_mesh.get_mut(*child) else {
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

        *vis_ray_mesh = Visibility::Visible;

        if !ray.vfx_started {
            ray.vfx_started = true;
            ev_damage_particles.send(DamageParticlesEvent {
                position: t - dir * 0.5,
                normal: -dir,
                color,
            });
        }

        ev_take_damage.send(TakeDamageEvent {
            target: ray.target,
            damage: time.delta_seconds() * dps,
        });
    }
}

fn laser_ray_despawn(
    time: Res<Time>,
    q_ray: Query<(Entity, &LaserRay)>,
    mut q_laser: Query<&mut LaserState>,
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

fn cleanup_laser_rays(q_rays: Query<Entity, With<LaserRay>>, mut cmd: Commands) {
    for entity in &q_rays {
        cmd.entity(entity).despawn_recursive();
    }
}
