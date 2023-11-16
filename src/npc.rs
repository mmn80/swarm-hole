use std::sync::Arc;

use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;

use crate::{
    app::{AppState, RunState},
    debug_ui::{DebugUiCommand, DebugUiEvent},
    physics::{Layer, ALL_LAYERS},
    player::Player,
    skills::{health::Health, laser::LaserConfig, melee::MeleeConfig, AddSkillEvent, Skill},
};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Npc>()
            .add_event::<SpawnNpcEvent>()
            .init_resource::<NonPlayerCharacters>()
            .add_systems(Startup, setup_npcs)
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
                (spawn_npc, spawn_random_npcs, move_npcs).run_if(in_state(AppState::Run)),
            );
    }
}

#[derive(Resource, Default)]
pub struct NonPlayerCharacters {
    pub npcs: Vec<Arc<NonPlayerCharacter>>,
    pub xp_drop_small: Handle<StandardMaterial>,
    pub xp_drop_big: Handle<StandardMaterial>,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Reflect)]
pub enum NonPlayerCharacterId {
    CortezLimonero,
    LuzTomatera,
}

#[derive(Reflect)]
pub struct NonPlayerCharacter {
    pub id: NonPlayerCharacterId,
    pub hp: u32,
    pub speed: f32,
    pub radius: f32,
    pub xp_drop: u32,
    pub skills: Vec<Skill>,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub frequency: f32,
}

#[derive(Component, Reflect)]
pub struct Npc {
    pub id: Arc<NonPlayerCharacter>,
}

fn setup_npcs(
    mut npcs: ResMut<NonPlayerCharacters>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    npcs.npcs = vec![
        Arc::new(NonPlayerCharacter {
            id: NonPlayerCharacterId::CortezLimonero,
            hp: 1,
            speed: 2.,
            radius: 0.5,
            xp_drop: 1,
            skills: vec![Skill::Melee(MeleeConfig { range: 1., dps: 3 })],
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
        }),
        Arc::new(NonPlayerCharacter {
            id: NonPlayerCharacterId::LuzTomatera,
            hp: 10,
            speed: 1.5,
            radius: 1.,
            xp_drop: 10,
            skills: vec![
                Skill::Melee(MeleeConfig { range: 1.5, dps: 3 }),
                Skill::Laser(LaserConfig {
                    range: 10.,
                    dps: 5.,
                    duration: 0.2,
                    cooldown: 1.,
                }),
            ],
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
        }),
    ];

    npcs.xp_drop_small = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 4.0, 1.0),
        metallic: 0.8,
        perceptual_roughness: 0.4,
        reflectance: 0.9,
        ..default()
    });
    npcs.xp_drop_big = materials.add(StandardMaterial {
        base_color: Color::rgb(4.0, 1.0, 1.0),
        metallic: 0.8,
        perceptual_roughness: 0.4,
        reflectance: 0.9,
        ..default()
    });
}

#[derive(Event)]
pub struct SpawnNpcEvent {
    pub character: Arc<NonPlayerCharacter>,
    pub location: Vec2,
}

fn spawn_npc(
    mut run_state: ResMut<RunState>,
    mut ev_spawn_npc: EventReader<SpawnNpcEvent>,
    mut ev_add_skill: EventWriter<AddSkillEvent>,
    mut cmd: Commands,
) {
    for ev in ev_spawn_npc.read() {
        let npc = &ev.character;
        let id = cmd
            .spawn((
                Npc { id: npc.clone() },
                Health(npc.hp as f32),
                PbrBundle {
                    transform: Transform::from_xyz(ev.location.x, npc.radius + 0.02, ev.location.y),
                    mesh: npc.mesh.clone(),
                    material: npc.material.clone(),
                    ..default()
                },
                RigidBody::Kinematic,
                Collider::ball(npc.radius),
                CollisionLayers::new([Layer::NPC], ALL_LAYERS),
            ))
            .id();
        for skill in &npc.skills {
            ev_add_skill.send(AddSkillEvent {
                skill: *skill,
                agent: id,
            });
        }
        cmd.entity(id)
            .insert(Name::new(format!("NPC {:?} ({id:?})", npc.id)));

        run_state.live_npcs += 1;
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
    mut ev_spawn_npc: EventWriter<SpawnNpcEvent>,
) {
    for ev in ev_debug_ui.read() {
        if ev.command == DebugUiCommand::SpawnNpcs {
            let count = ev.param;
            info!("spawning {count} NPCs...");

            let mut rng = thread_rng();
            let npc_idx = WeightedIndex::new(npcs.npcs.iter().map(|item| item.frequency)).unwrap();

            let w = ((count as f32).sqrt() / 2.).ceil() as i32;
            let dist = (NPC_DIST - 4.) / 2.;
            let mut n = 0;
            for xi in -w..=w {
                for zi in -w..=w {
                    let npc_type = &npcs.npcs[npc_idx.sample(&mut rng)];
                    let x = xi as f32 * NPC_DIST + rng.gen_range(-dist..dist);
                    let z = zi as f32 * NPC_DIST + rng.gen_range(-dist..dist);

                    ev_spawn_npc.send(SpawnNpcEvent {
                        character: npc_type.clone(),
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
        let dir = Vec2::new(player_pos.x - npc_pos.x, player_pos.z - npc_pos.z).normalize()
            * npc.id.speed;
        lin_vel.x = dir.x;
        lin_vel.z = dir.y;
    }
}

fn cleanup_npcs(q_npc: Query<Entity, With<Npc>>, mut cmd: Commands) {
    for entity in &q_npc {
        cmd.entity(entity).despawn_recursive();
    }
}
