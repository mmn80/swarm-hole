use bevy::{ecs::system::Command, prelude::*};
use bevy_xpbd_3d::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;

use crate::{
    app::{AppState, RunState},
    debug_ui::{DebugUiCommand, DebugUiEvent},
    physics::{Layer, ALL_LAYERS},
    player::Player,
    skills::{health::Health, laser::LaserConfig, melee::MeleeConfig},
};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NonPlayerCharacter>()
            .init_resource::<Npcs>()
            .add_systems(Startup, setup_npc_resources)
            .add_systems(OnEnter(AppState::Cleanup), cleanup_npcs)
            .add_systems(
                OnTransition {
                    from: AppState::Menu,
                    to: AppState::Run,
                },
                spawn_start_npcs,
            )
            .add_systems(
                Update,
                (spawn_random_npcs, move_npcs).run_if(in_state(AppState::Run)),
            );
    }
}

#[derive(Resource, Default)]
pub struct Npcs {
    pub fst_mesh: Handle<Mesh>,
    pub fst_mat: Handle<StandardMaterial>,
    pub snd_mesh: Handle<Mesh>,
    pub snd_mat: Handle<StandardMaterial>,
}

fn setup_npc_resources(
    mut npc_resources: ResMut<Npcs>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    npc_resources.fst_mesh = meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 0.5,
            subdivisions: 5,
        })
        .unwrap(),
    );
    npc_resources.fst_mat = materials.add(StandardMaterial {
        base_color: Color::LIME_GREEN,
        metallic: 0.8,
        perceptual_roughness: 0.3,
        ..default()
    });
    npc_resources.snd_mesh = meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 1.,
            subdivisions: 8,
        })
        .unwrap(),
    );
    npc_resources.snd_mat = materials.add(StandardMaterial {
        base_color: Color::TOMATO,
        metallic: 0.8,
        perceptual_roughness: 0.3,
        ..default()
    });
}

#[derive(Component, Reflect, Clone)]
pub struct NonPlayerCharacter {
    pub hp: u32,
    pub speed: f32,
    pub radius: f32,
    pub xp_drop: u32,
    pub frequency: f32,
}

const CORTEZ_LIMONERO: &str = "cortez_limonero";
const LUZ_TOMATARA: &str = "luz_tomatera";

pub struct SpawnNpc {
    pub character: String,
    pub location: Vec2,
}

impl Command for SpawnNpc {
    fn apply(self, world: &mut World) {
        let Some((fst_mesh, fst_mat, snd_mesh, snd_mat)) = ({
            if let Some(npc_resources) = world.get_resource::<Npcs>() {
                Some((
                    npc_resources.fst_mesh.clone(),
                    npc_resources.fst_mat.clone(),
                    npc_resources.snd_mesh.clone(),
                    npc_resources.snd_mat.clone(),
                ))
            } else {
                None
            }
        }) else {
            return;
        };

        if let Some((npc, mesh, material)) = {
            if self.character == CORTEZ_LIMONERO {
                Some((
                    NonPlayerCharacter {
                        hp: 1,
                        speed: 2.,
                        radius: 0.5,
                        xp_drop: 1,
                        frequency: 1.,
                    },
                    fst_mesh,
                    fst_mat,
                ))
            } else if self.character == LUZ_TOMATARA {
                Some((
                    NonPlayerCharacter {
                        hp: 10,
                        speed: 1.5,
                        radius: 1.,
                        xp_drop: 10,
                        frequency: 0.1,
                    },
                    snd_mesh,
                    snd_mat,
                ))
            } else {
                None
            }
        } {
            let id = world
                .spawn((
                    npc.clone(),
                    Health(npc.hp as f32),
                    PbrBundle {
                        transform: Transform::from_xyz(
                            self.location.x,
                            npc.radius + 0.02,
                            self.location.y,
                        ),
                        mesh,
                        material,
                        ..default()
                    },
                    RigidBody::Kinematic,
                    Collider::ball(npc.radius),
                    CollisionLayers::new([Layer::NPC], ALL_LAYERS),
                ))
                .id();
            if self.character == CORTEZ_LIMONERO {
                world
                    .entity_mut(id)
                    .insert(MeleeConfig { range: 1., dps: 3 });
            } else if self.character == LUZ_TOMATARA {
                world.entity_mut(id).insert((
                    MeleeConfig { range: 1.5, dps: 3 },
                    LaserConfig {
                        range: 10.,
                        dps: 5.,
                        duration: 0.2,
                        cooldown: 1.,
                    },
                ));
            }

            world
                .entity_mut(id)
                .insert(Name::new(format!("NPC {:?} ({id:?})", self.character)));

            if let Some(mut run_state) = world.get_resource_mut::<RunState>() {
                run_state.live_npcs += 1;
            };
        }
    }
}

fn spawn_start_npcs(mut ev_debug_ui: EventWriter<DebugUiEvent>) {
    ev_debug_ui.send(DebugUiEvent {
        command: DebugUiCommand::SpawnNpcs,
        param: 250,
    });
}

const NPCS: [(&str, f32); 2] = [(CORTEZ_LIMONERO, 1.), (LUZ_TOMATARA, 0.1)];
const NPC_DIST: f32 = 30.0;

fn spawn_random_npcs(mut ev_debug_ui: EventReader<DebugUiEvent>, mut cmd: Commands) {
    for ev in ev_debug_ui.read() {
        if ev.command == DebugUiCommand::SpawnNpcs {
            let count = ev.param;
            info!("spawning {count} NPCs...");

            let mut rng = thread_rng();
            let npc_idx = WeightedIndex::new(NPCS.iter().map(|(_, frequency)| frequency)).unwrap();

            let w = ((count as f32).sqrt() / 2.).ceil() as i32;
            let dist = (NPC_DIST - 4.) / 2.;
            let mut n = 0;
            for xi in -w..=w {
                for zi in -w..=w {
                    let npc = NPCS[npc_idx.sample(&mut rng)].0;
                    let x = xi as f32 * NPC_DIST + rng.gen_range(-dist..dist);
                    let z = zi as f32 * NPC_DIST + rng.gen_range(-dist..dist);

                    cmd.add(SpawnNpc {
                        character: npc.to_string(),
                        location: Vec2::new(x, z),
                    });

                    n += 1;
                    if n == count {
                        break;
                    }
                }
                if n == count {
                    break;
                }
            }
            break;
        }
    }
}

fn move_npcs(
    mut q_npc: Query<(&NonPlayerCharacter, &Position, &mut LinearVelocity)>,
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

fn cleanup_npcs(q_npc: Query<Entity, With<NonPlayerCharacter>>, mut cmd: Commands) {
    for entity in &q_npc {
        cmd.entity(entity).despawn_recursive();
    }
}
