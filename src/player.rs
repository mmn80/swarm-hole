use bevy::prelude::*;
use bevy_xpbd_3d::{math::*, prelude::*, PhysicsSchedule, PhysicsStepSet};

use crate::{
    camera::MainCameraFocusEvent,
    debug_ui::DebugUi,
    materials::BasicMaterials,
    npc::{Health, XpDrop},
    physics::{Layer, ALL_LAYERS},
    weapons::laser::Laser,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_systems(Startup, setup_player)
            .add_systems(Update, gather_xp)
            .add_systems(
                PhysicsSchedule,
                move_player.before(PhysicsStepSet::BroadPhase),
            );
    }
}

#[derive(Component, Reflect)]
pub struct Player {
    pub speed: f32,
    pub xp: u32,
    pub gather_range: f32,
}

fn setup_player(
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<BasicMaterials>,
    mut cmd: Commands,
) {
    let player_height = 2.;
    let player_width = 0.3;
    let cap_h = player_height - 2. * player_width;

    let id = cmd
        .spawn((
            Player {
                speed: 4.,
                xp: 0,
                gather_range: 3.,
            },
            Health(100.),
            Laser::new(15., 20., 0.5, 0.5, true),
            PbrBundle {
                transform: Transform::from_xyz(0.0, player_height / 2. + 0.2, 0.0),
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: player_width,
                    rings: 0,
                    depth: cap_h,
                    latitudes: 16,
                    longitudes: 32,
                    uv_profile: shape::CapsuleUvProfile::Aspect,
                })),
                material: materials.player.clone(),
                ..default()
            },
            RigidBody::Kinematic,
            Collider::capsule(cap_h, player_width),
            CollisionLayers::new([Layer::Player], ALL_LAYERS),
            ShapeCaster::new(
                Collider::capsule(cap_h - 0.1, player_width - 0.05),
                Vector::ZERO,
                Quaternion::default(),
                Vector::NEG_Y,
            )
            .with_max_time_of_impact(0.11)
            .with_max_hits(1),
        ))
        .id();
    cmd.entity(id).insert(Name::new(format!("Player ({id:?})")));
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

const XP_ATTRACT_ACC: f32 = 50.;

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
                &Collider::ball(player.gather_range),
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
                    delta = delta.normalize() * (old_speed + time.delta_seconds() * XP_ATTRACT_ACC);
                    lin_vel.x = delta.x;
                    lin_vel.z = delta.z;
                }
            }
        }
    }
}
