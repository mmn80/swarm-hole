use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;

use crate::{
    debug_ui::{DebugUiCommand, DebugUiEvent},
    physics::{Layer, ALL_LAYERS},
    player::Player,
};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Npc>()
            .init_resource::<NpcCommons>()
            .add_systems(Startup, setup_npc_commons)
            .add_systems(Update, (spawn_npcs, move_npcs, die));
    }
}

#[derive(Resource, Default)]
pub struct NpcCommons {
    pub npc_types: Vec<NpcType>,
}

pub struct NpcType {
    pub hp: u32,
    pub speed: f32,
    pub radius: f32,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub frequency: f32,
}

#[derive(Component, Reflect)]
pub struct Npc {
    pub speed: f32,
}

fn setup_npc_commons(
    mut npcs: ResMut<NpcCommons>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    npcs.npc_types = vec![
        NpcType {
            hp: 1,
            speed: 2.,
            radius: 0.5,
            mesh: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: 0.5,
                    subdivisions: 5,
                })
                .unwrap(),
            ),
            material: materials.add(StandardMaterial {
                base_color: Color::PURPLE,
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
            mesh: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: 1.,
                    subdivisions: 8,
                })
                .unwrap(),
            ),
            material: materials.add(StandardMaterial {
                base_color: Color::TURQUOISE,
                metallic: 0.8,
                perceptual_roughness: 0.3,
                ..default()
            }),
            frequency: 0.1,
        },
    ];
}

const NPC_DIST: f32 = 5.0;

fn spawn_npcs(
    npcs: Res<NpcCommons>,
    mut ev_debug_ui: EventReader<DebugUiEvent>,
    mut cmd: Commands,
) {
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
                            },
                            Health(npc_type.hp),
                            PbrBundle {
                                transform: Transform::from_xyz(x, npc_type.radius + 0.1, z),
                                mesh: npc_type.mesh.clone(),
                                material: npc_type.material.clone(),
                                ..default()
                            },
                            RigidBody::Kinematic,
                            Collider::ball(npc_type.radius),
                            CollisionLayers::new([Layer::NPC], ALL_LAYERS),
                        ))
                        .id();
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
pub struct Health(pub u32);

impl Health {
    pub fn take_damage(&mut self, damage: u32) {
        self.0 = if damage >= self.0 { 0 } else { self.0 - damage };
    }
}

fn die(q_npc: Query<(Entity, &Health)>, mut cmd: Commands) {
    for (npc_ent, health) in &q_npc {
        if health.0 <= 0 {
            cmd.entity(npc_ent).despawn_recursive();
        }
    }
}
