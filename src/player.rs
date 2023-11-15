use std::sync::Arc;

use bevy::prelude::*;
use bevy_xpbd_3d::{math::*, prelude::*, PhysicsSchedule, PhysicsStepSet};

use crate::{
    app::AppState,
    camera::MainCameraFocusEvent,
    debug_ui::DebugUi,
    npc::{Health, XpDrop},
    physics::{Layer, ALL_LAYERS},
    skills::{laser::LaserConfig, AddSkillEvent, Skill},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_event::<SpawnPlayerEvent>()
            .init_resource::<PlayerCharacters>()
            .add_systems(Startup, setup_player_characters)
            .add_systems(
                OnTransition {
                    from: AppState::Menu,
                    to: AppState::Run,
                },
                spawn_main_player,
            )
            .add_systems(
                OnTransition {
                    from: AppState::Paused,
                    to: AppState::Menu,
                },
                cleanup_players,
            )
            .add_systems(
                Update,
                (spawn_player, gather_xp, regen_health).run_if(in_state(AppState::Run)),
            )
            .add_systems(
                PhysicsSchedule,
                move_player
                    .before(PhysicsStepSet::BroadPhase)
                    .run_if(in_state(AppState::Run)),
            );
    }
}

#[derive(Resource, Default)]
pub struct PlayerCharacters {
    pub characters: Vec<Arc<PlayerCharacter>>,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Reflect)]
pub enum PlayerCharacterId {
    JoseCapsulado,
}

#[derive(Reflect)]
pub struct PlayerCharacter {
    pub id: PlayerCharacterId,
    pub hp: u32,
    pub speed: f32,
    pub width: f32,
    pub height: f32,
    pub gather_range: f32,
    pub gather_acceleration: f32,
    pub hp_regen_per_sec: f32,
    pub skills: Vec<Skill>,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

#[derive(Component, Reflect)]
pub struct Player {
    pub id: Arc<PlayerCharacter>,
    pub xp: u32,
}

fn setup_player_characters(
    mut pcs: ResMut<PlayerCharacters>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (height, width) = (2., 0.3);
    let cap_h = height - 2. * width;
    let pc = Arc::new(PlayerCharacter {
        id: PlayerCharacterId::JoseCapsulado,
        hp: 100,
        speed: 4.,
        width,
        height,
        gather_range: 3.,
        gather_acceleration: 50.,
        hp_regen_per_sec: 1.,
        skills: vec![Skill::Laser(LaserConfig {
            range: 15.,
            dps: 20.,
            duration: 0.5,
            cooldown: 0.5,
        })],
        mesh: meshes.add(Mesh::from(shape::Capsule {
            radius: width,
            rings: 0,
            depth: cap_h,
            latitudes: 16,
            longitudes: 32,
            uv_profile: shape::CapsuleUvProfile::Aspect,
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            metallic: 0.0,
            perceptual_roughness: 0.5,
            ..default()
        }),
    });
    pcs.characters = vec![pc];
}

fn spawn_main_player(
    pcs: Res<PlayerCharacters>,
    mut ev_spawn_player: EventWriter<SpawnPlayerEvent>,
) {
    ev_spawn_player.send(SpawnPlayerEvent {
        character: pcs.characters[0].clone(),
        location: Vec2::ZERO,
    });
}

fn cleanup_players(q_player: Query<Entity, With<Player>>, mut cmd: Commands) {
    for entity in &q_player {
        cmd.entity(entity).despawn_recursive();
    }
}

#[derive(Event)]
pub struct SpawnPlayerEvent {
    pub character: Arc<PlayerCharacter>,
    pub location: Vec2,
}

fn spawn_player(
    mut ev_spawn_player: EventReader<SpawnPlayerEvent>,
    mut ev_add_skill: EventWriter<AddSkillEvent>,
    mut cmd: Commands,
) {
    for ev in ev_spawn_player.read() {
        let pc = &ev.character;
        let cap_h = pc.height - 2. * pc.width;
        let id = cmd
            .spawn((
                Player {
                    id: pc.clone(),
                    xp: 0,
                },
                Health(pc.hp as f32),
                PbrBundle {
                    transform: Transform::from_xyz(
                        ev.location.x,
                        pc.height / 2. + 0.2,
                        ev.location.y,
                    ),
                    mesh: pc.mesh.clone(),
                    material: pc.material.clone(),
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
        for skill in &pc.skills {
            ev_add_skill.send(AddSkillEvent {
                skill: *skill,
                parent: id,
            });
        }
        cmd.entity(id)
            .insert(Name::new(format!("Player {:?} ({id:?})", pc.id)));
    }
}

const PLAYER_ACC_STEPS: f32 = 10.;

fn move_player(
    keyboard: Res<Input<KeyCode>>,
    debug_ui: Res<DebugUi>,
    mut q_player: Query<(&Transform, &Player, &mut LinearVelocity, &ShapeHits)>,
    mut ev_refocus: EventWriter<MainCameraFocusEvent>,
) {
    for (player_tr, player, mut linear_velocity, ground_hits) in &mut q_player {
        if !ground_hits.is_empty() {
            linear_velocity.y = 0.0;
        } else {
            linear_velocity.y -= 0.4;
        }

        let acc = player.id.speed / PLAYER_ACC_STEPS;
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
        vel = vel.clamp_length_max(player.id.speed);

        linear_velocity.x = vel.x;
        linear_velocity.z = vel.y;
        linear_velocity.y *= 0.98;

        ev_refocus.send(MainCameraFocusEvent {
            focus: player_tr.translation,
        });
    }
}

fn gather_xp(
    time: Res<Time>,
    q_space: SpatialQuery,
    mut q_player: Query<(&Transform, &mut Player)>,
    mut q_xp_drop: Query<(Entity, &Transform, &mut LinearVelocity, &XpDrop)>,
    mut cmd: Commands,
) {
    for (tr_player, mut player) in &mut q_player {
        for ent in q_space
            .shape_intersections(
                &Collider::ball(player.id.gather_range),
                tr_player.translation,
                Quat::default(),
                SpatialQueryFilter::new().with_masks([Layer::Building]),
            )
            .iter()
        {
            if let Ok((ent, tr_xp, mut lin_vel, xp_drop)) = q_xp_drop.get_mut(*ent) {
                let mut delta = tr_player.translation - tr_xp.translation;
                if delta.length() < XpDrop::get_height(xp_drop.0) + 1. {
                    player.xp += xp_drop.0;
                    cmd.entity(ent).despawn_recursive();
                } else {
                    lin_vel.y = 0.;
                    let old_speed = lin_vel.length();
                    delta.y = 0.;
                    delta = delta.normalize()
                        * (old_speed + time.delta_seconds() * player.id.gather_acceleration);
                    lin_vel.x = delta.x;
                    lin_vel.z = delta.z;
                }
            }
        }
    }
}

fn regen_health(time: Res<Time>, mut q_player: Query<(&mut Health, &Player)>) {
    for (mut health, player) in &mut q_player {
        health.0 =
            (health.0 + player.id.hp_regen_per_sec * time.delta_seconds()).min(player.id.hp as f32);
    }
}
