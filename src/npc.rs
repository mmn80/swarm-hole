use bevy::{ecs::system::Command, prelude::*};
use bevy_xpbd_3d::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;

use crate::{
    app::{AppState, RunState},
    debug_ui::{DebugUiCommand, DebugUiEvent},
    physics::{Layer, ALL_LAYERS},
    player::Player,
    skills::{health::Health, init_skills, laser::Laser, melee::Melee, Skill},
};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NonPlayerCharacter>()
            .init_resource::<NpcHandles>()
            .init_resource::<NonPlayerCharacters>()
            .add_systems(Startup, (setup_npc_handles, setup_npcs))
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
pub struct NpcHandles {
    pub meshes: Vec<Handle<Mesh>>,
    pub materials: Vec<Handle<StandardMaterial>>,
}

fn setup_npc_handles(
    mut npc_handles: ResMut<NpcHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    npc_handles.meshes = vec![
        meshes.add(
            Mesh::try_from(shape::Icosphere {
                radius: 0.5,
                subdivisions: 5,
            })
            .unwrap(),
        ),
        meshes.add(
            Mesh::try_from(shape::Icosphere {
                radius: 1.,
                subdivisions: 8,
            })
            .unwrap(),
        ),
    ];
    npc_handles.materials = vec![
        materials.add(StandardMaterial {
            base_color: Color::LIME_GREEN,
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::TOMATO,
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        }),
    ];
}

#[derive(Resource, Default)]
pub struct NonPlayerCharacters(pub Vec<NonPlayerCharacter>);

#[derive(Reflect, Clone)]
pub struct NonPlayerCharacter {
    pub name: String,
    pub max_hp: u32,
    pub xp_drop: u32,
    pub speed: f32,
    pub frequency: f32,
    pub radius: f32,
    pub mesh_idx: usize,
    pub material_idx: usize,
    pub skills: Vec<Skill>,
}

fn setup_npcs(mut npcs: ResMut<NonPlayerCharacters>) {
    npcs.0 = vec![
        NonPlayerCharacter {
            name: "Cortez Limonero".to_string(),
            max_hp: 1,
            xp_drop: 1,
            speed: 2.,
            radius: 0.5,
            frequency: 1.,
            mesh_idx: 0,
            material_idx: 0,
            skills: vec![Skill::Melee(vec![Melee { range: 1., dps: 3 }])],
        },
        NonPlayerCharacter {
            name: "Luz Tomatera".to_string(),
            max_hp: 10,
            xp_drop: 10,
            speed: 1.5,
            radius: 1.,
            frequency: 0.1,
            mesh_idx: 1,
            material_idx: 1,
            skills: vec![
                Skill::Melee(vec![Melee { range: 1.5, dps: 3 }]),
                Skill::Laser(vec![Laser {
                    range: 10.,
                    dps: 5.,
                    duration: 0.2,
                    cooldown: 1.,
                }]),
            ],
        },
    ];
}

#[derive(Component, Reflect, Clone)]
pub struct Npc {
    pub max_hp: u32,
    pub xp_drop: u32,
    pub speed: f32,
}

pub struct SpawnNpc {
    pub character: NonPlayerCharacter,
    pub location: Vec2,
}

impl Command for SpawnNpc {
    fn apply(self, world: &mut World) {
        {
            let Some(npc_handles) = world.get_resource::<NpcHandles>() else {
                return;
            };
            let npc = &self.character;
            let id = world
                .spawn((
                    Npc {
                        max_hp: npc.max_hp,
                        xp_drop: npc.xp_drop,
                        speed: npc.speed,
                    },
                    Health(npc.max_hp as f32),
                    PbrBundle {
                        transform: Transform::from_xyz(
                            self.location.x,
                            npc.radius + 0.02,
                            self.location.y,
                        ),
                        mesh: npc_handles.meshes.get(npc.mesh_idx).unwrap().clone(),
                        material: npc_handles.materials.get(npc.material_idx).unwrap().clone(),
                        ..default()
                    },
                    RigidBody::Kinematic,
                    Collider::ball(npc.radius),
                    CollisionLayers::new([Layer::NPC], ALL_LAYERS),
                ))
                .id();
            init_skills(id, &npc.skills, world);
            world
                .entity_mut(id)
                .insert(Name::new(format!("NPC {:?} ({id:?})", npc.name)));
        }

        if let Some(mut run_state) = world.get_resource_mut::<RunState>() {
            run_state.live_npcs += 1;
        };
    }
}

fn spawn_start_npcs(mut ev_debug_ui: EventWriter<DebugUiEvent>) {
    ev_debug_ui.send(DebugUiEvent {
        command: DebugUiCommand::SpawnNpcs,
        param: 250,
    });
}

const NPC_DIST: f32 = 30.0;

fn spawn_random_npcs(
    npcs: Res<NonPlayerCharacters>,
    mut ev_debug_ui: EventReader<DebugUiEvent>,
    mut cmd: Commands,
) {
    for ev in ev_debug_ui.read() {
        if ev.command == DebugUiCommand::SpawnNpcs {
            let count = ev.param;
            info!("spawning {count} NPCs...");

            let mut rng = thread_rng();
            let npc_idx = WeightedIndex::new(npcs.0.iter().map(|npc| npc.frequency)).unwrap();

            let w = ((count as f32).sqrt() / 2.).ceil() as i32;
            let dist = (NPC_DIST - 4.) / 2.;
            let mut n = 0;
            for xi in -w..=w {
                for zi in -w..=w {
                    let npc = &npcs.0[npc_idx.sample(&mut rng)];
                    let x = xi as f32 * NPC_DIST + rng.gen_range(-dist..dist);
                    let z = zi as f32 * NPC_DIST + rng.gen_range(-dist..dist);

                    cmd.add(SpawnNpc {
                        character: npc.clone(),
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

fn cleanup_npcs(q_npc: Query<Entity, With<Npc>>, mut cmd: Commands) {
    for entity in &q_npc {
        cmd.entity(entity).despawn_recursive();
    }
}
