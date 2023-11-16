use bevy::{ecs::system::Command, prelude::*};
use bevy_xpbd_3d::{math::*, prelude::*, PhysicsSchedule, PhysicsStepSet};

use crate::{
    app::AppState,
    camera::MainCameraFocusEvent,
    debug_ui::DebugUi,
    physics::{Layer, ALL_LAYERS},
    skills::{health::Health, laser::LaserConfig},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .init_resource::<PlayerResources>()
            .add_systems(Startup, setup_player_resources)
            .add_systems(
                OnTransition {
                    from: AppState::Menu,
                    to: AppState::Run,
                },
                spawn_main_player,
            )
            .add_systems(OnEnter(AppState::Cleanup), cleanup_players)
            .add_systems(
                PhysicsSchedule,
                move_player
                    .before(PhysicsStepSet::BroadPhase)
                    .run_if(in_state(AppState::Run)),
            );
    }
}

#[derive(Resource, Default)]
pub struct PlayerResources {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

fn setup_player_resources(
    mut player_resources: ResMut<PlayerResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (height, width) = (2., 0.3);
    let cap_h = height - 2. * width;
    player_resources.mesh = meshes.add(Mesh::from(shape::Capsule {
        radius: width,
        rings: 0,
        depth: cap_h,
        latitudes: 16,
        longitudes: 32,
        uv_profile: shape::CapsuleUvProfile::Aspect,
    }));
    player_resources.material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        metallic: 0.0,
        perceptual_roughness: 0.5,
        ..default()
    });
}

#[derive(Component, Reflect, Clone)]
pub struct PlayerCharacter {
    pub hp: u32,
    pub speed: f32,
    pub width: f32,
    pub height: f32,
    pub gather_range: f32,
    pub gather_acceleration: f32,
    pub hp_regen_per_sec: f32,
}

#[derive(Component, Reflect)]
pub struct Player {
    pub xp: u32,
}

const JOSE_CAPSULADO: &str = "jose_capsulado";

fn spawn_main_player(mut cmd: Commands) {
    cmd.add(SpawnPlayer {
        character: JOSE_CAPSULADO.to_string(),
        location: Vec2::ZERO,
    });
}

pub struct SpawnPlayer {
    pub character: String,
    pub location: Vec2,
}

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        let Some(player_resources) = world.get_resource::<PlayerResources>() else {
            return;
        };
        if let Some(pc) = {
            if self.character == JOSE_CAPSULADO {
                Some(PlayerCharacter {
                    hp: 100,
                    speed: 4.,
                    width: 0.3,
                    height: 2.,
                    gather_range: 5.,
                    gather_acceleration: 50.,
                    hp_regen_per_sec: 1.,
                })
            } else {
                None
            }
        } {
            let cap_h = pc.height - 2. * pc.width;
            let id = world
                .spawn((
                    pc.clone(),
                    Player { xp: 0 },
                    Health(pc.hp as f32),
                    PbrBundle {
                        transform: Transform::from_xyz(
                            self.location.x,
                            pc.height / 2. + 0.2,
                            self.location.y,
                        ),
                        mesh: player_resources.mesh.clone(),
                        material: player_resources.material.clone(),
                        ..default()
                    },
                    RigidBody::Kinematic,
                    Collider::capsule(cap_h, pc.width),
                    CollisionLayers::new([Layer::Player], ALL_LAYERS),
                    ShapeCaster::new(
                        Collider::capsule(cap_h - 0.1, pc.width - 0.05),
                        Vector::ZERO,
                        Quaternion::default(),
                        Vector::NEG_Y,
                    )
                    .with_max_time_of_impact(0.11)
                    .with_max_hits(1),
                ))
                .id();
            if self.character == JOSE_CAPSULADO {
                world.entity_mut(id).insert(LaserConfig {
                    range: 15.,
                    dps: 20.,
                    duration: 0.5,
                    cooldown: 0.5,
                });
            }
            world
                .entity_mut(id)
                .insert(Name::new(format!("Player {} ({id:?})", self.character)));
        }
    }
}

const PLAYER_ACC_STEPS: f32 = 10.;

fn move_player(
    keyboard: Res<Input<KeyCode>>,
    debug_ui: Res<DebugUi>,
    mut q_player: Query<(
        &Transform,
        &PlayerCharacter,
        &mut LinearVelocity,
        &ShapeHits,
    )>,
    mut ev_refocus: EventWriter<MainCameraFocusEvent>,
) {
    for (player_tr, player, mut linear_velocity, ground_hits) in &mut q_player {
        if !ground_hits.is_empty() {
            linear_velocity.y = 0.0;
        } else {
            linear_velocity.y -= 0.4;
        }

        let acc = player.speed / PLAYER_ACC_STEPS;
        let mut vel = Vec2::new(linear_velocity.x, linear_velocity.z);
        if !debug_ui.has_focus() {
            let mut changed = false;
            if keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up) {
                vel.y -= acc;
                changed = true;
            }
            if keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left) {
                vel.x -= acc;
                changed = true;
            }
            if keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down) {
                vel.y += acc;
                changed = true;
            }
            if keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right) {
                vel.x += acc;
                changed = true;
            }
            if !changed {
                vel *= 0.8;
            }
            if keyboard.just_pressed(KeyCode::Space) && !ground_hits.is_empty() {
                linear_velocity.y += 20.0;
            }
        }
        vel = vel.clamp_length_max(player.speed);

        linear_velocity.x = vel.x;
        linear_velocity.z = vel.y;
        linear_velocity.y *= 0.98;

        ev_refocus.send(MainCameraFocusEvent {
            focus: player_tr.translation,
        });
    }
}

fn cleanup_players(q_player: Query<Entity, With<Player>>, mut cmd: Commands) {
    for entity in &q_player {
        cmd.entity(entity).despawn_recursive();
    }
}
