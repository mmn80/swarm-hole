use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;

use crate::{
    debug_ui::{DebugUiCommand, DebugUiEvent},
    materials::BasicMaterials,
    physics::{Layer, ALL_LAYERS},
    player::Player,
    weapons::{laser::Laser, melee::Melee},
};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Npc>()
            .init_resource::<Npcs>()
            .add_systems(Startup, setup_npcs)
            .add_systems(Update, (spawn_npcs, move_npcs, slow_xp_drops, die));
    }
}

#[derive(Resource, Default)]
pub struct Npcs {
    pub npc_types: Vec<NpcType>,
}

pub struct NpcType {
    pub hp: u32,
    pub speed: f32,
    pub radius: f32,
    pub has_laser: bool,
    pub melee_dps: u32,
    pub xp_drop: u32,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub frequency: f32,
}

#[derive(Component, Reflect)]
pub struct Npc {
    pub speed: f32,
    pub xp_drop: u32,
}

fn setup_npcs(
    mut npcs: ResMut<Npcs>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    npcs.npc_types = vec![
        NpcType {
            hp: 1,
            speed: 2.,
            radius: 0.5,
            has_laser: false,
            melee_dps: 3,
            xp_drop: 1,
            mesh: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: 0.5,
                    subdivisions: 5,
                })
                .unwrap(),
            ),
            material: materials.add(StandardMaterial {
                base_color: Color::LIME_GREEN,
                metallic: 0.8,
                perceptual_roughness: 0.3,
                ..default()
            }),
            frequency: 1.,
        },
        NpcType {
            hp: 10,
            speed: 1.5,
            radius: 1.,
            has_laser: true,
            melee_dps: 3,
            xp_drop: 10,
            mesh: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: 1.,
                    subdivisions: 8,
                })
                .unwrap(),
            ),
            material: materials.add(StandardMaterial {
                base_color: Color::TOMATO,
                metallic: 0.8,
                perceptual_roughness: 0.3,
                ..default()
            }),
            frequency: 0.1,
        },
    ];
}

const NPC_DIST: f32 = 10.0;

fn spawn_npcs(npcs: Res<Npcs>, mut ev_debug_ui: EventReader<DebugUiEvent>, mut cmd: Commands) {
    for ev in ev_debug_ui.read() {
        if ev.command == DebugUiCommand::SpawnNpcs {
            let count = ev.param;
            info!("spawning {count} NPCs...");

            let mut rng = thread_rng();
            let npc_idx =
                WeightedIndex::new(npcs.npc_types.iter().map(|item| item.frequency)).unwrap();

            let w = ((count as f32).sqrt() / 2.).ceil() as i32;
            let dist = (NPC_DIST - 4.) / 2.;
            let mut n = 0;
            for xi in -w..=w {
                for zi in -w..=w {
                    let npc_type = &npcs.npc_types[npc_idx.sample(&mut rng)];
                    let x = xi as f32 * NPC_DIST + rng.gen_range(-dist..dist);
                    let z = zi as f32 * NPC_DIST + rng.gen_range(-dist..dist);
                    let id = cmd
                        .spawn((
                            Npc {
                                speed: npc_type.speed,
                                xp_drop: npc_type.xp_drop,
                            },
                            Health(npc_type.hp as f32),
                            PbrBundle {
                                transform: Transform::from_xyz(x, npc_type.radius + 0.02, z),
                                mesh: npc_type.mesh.clone(),
                                material: npc_type.material.clone(),
                                ..default()
                            },
                            RigidBody::Kinematic,
                            Collider::ball(npc_type.radius),
                            CollisionLayers::new([Layer::NPC], ALL_LAYERS),
                            Melee {
                                range: npc_type.radius + 0.5,
                                dps: npc_type.melee_dps,
                            },
                        ))
                        .id();
                    if npc_type.has_laser {
                        cmd.entity(id).insert(Laser::new(10., 5., 0.2, 1., false));
                    }
                    cmd.entity(id).insert(Name::new(format!("NPC ({id:?})")));

                    n += 1;
                    if n == count {
                        break;
                    }
                }
            }
            break;
        }
    }
}

fn move_npcs(
    mut q_npc: Query<(&Npc, &Position, &mut LinearVelocity)>,
    q_player: Query<&Position, With<Player>>,
) {
    let Ok(player_pos) = q_player.get_single() else {
        for (_, _, mut lin_vel) in &mut q_npc {
            lin_vel.x = 0.;
            lin_vel.y = 0.;
            lin_vel.z = 0.;
        }
        return;
    };
    for (npc, npc_pos, mut lin_vel) in &mut q_npc {
        lin_vel.y = 0.;
        let dir =
            Vec2::new(player_pos.x - npc_pos.x, player_pos.z - npc_pos.z).normalize() * npc.speed;
        lin_vel.x = dir.x;
        lin_vel.z = dir.y;
    }
}

#[derive(Component)]
pub struct Health(pub f32);

impl Health {
    pub fn take_damage(&mut self, damage: f32) {
        self.0 = if damage >= self.0 {
            0.
        } else {
            self.0 - damage
        };
    }
}

#[derive(Component)]
pub struct XpDrop(pub u32);

impl XpDrop {
    pub fn is_big(drop: u32) -> bool {
        drop > 5
    }

    pub fn get_height(drop: u32) -> f32 {
        if XpDrop::is_big(drop) {
            0.4
        } else {
            0.2
        }
    }
}

fn slow_xp_drops(time: Res<Time>, mut q_npc: Query<&mut LinearVelocity, With<XpDrop>>) {
    for mut lin_vel in &mut q_npc {
        let speed = lin_vel.length();
        if speed > f32::EPSILON {
            let dir = lin_vel.normalize_or_zero();
            lin_vel.0 = (speed - time.delta_seconds() * 5.).max(0.) * dir;
        }
    }
}

fn die(
    mut meshes: ResMut<Assets<Mesh>>,
    q_npc: Query<(Entity, &Health, &Transform, Option<&Npc>)>,
    materials: Res<BasicMaterials>,
    mut cmd: Commands,
) {
    for (npc_ent, health, tr_npc, npc) in &q_npc {
        if health.0 <= f32::EPSILON {
            if let Some(npc) = npc {
                let h = XpDrop::get_height(npc.xp_drop);
                let p = tr_npc.translation;
                let id = cmd
                    .spawn((
                        XpDrop(npc.xp_drop),
                        PbrBundle {
                            transform: Transform::from_translation(Vec3::new(p.x, h + 0.02, p.z)),
                            mesh: meshes.add(
                                Mesh::try_from(shape::Icosphere {
                                    radius: h,
                                    subdivisions: 4,
                                })
                                .unwrap(),
                            ),
                            material: (if XpDrop::is_big(npc.xp_drop) {
                                materials.xp_drop_big.clone()
                            } else {
                                materials.xp_drop_small.clone()
                            }),
                            ..default()
                        },
                        RigidBody::Kinematic,
                        Collider::ball(h),
                        CollisionLayers::new([Layer::Building], [Layer::Building, Layer::Player]),
                    ))
                    .id();
                cmd.entity(id)
                    .insert(Name::new(format!("Xp Drop of {} ({id:?})", npc.xp_drop)));
            }
            cmd.entity(npc_ent).despawn_recursive();
        }
    }
}
