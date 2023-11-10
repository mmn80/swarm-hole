use bevy::prelude::*;
use bevy_xpbd_3d::{math::*, prelude::*, PhysicsSchedule, PhysicsStepSet};

use crate::{
    camera::MainCameraFocusEvent,
    debug_ui::DebugUi,
    materials::BasicMaterials,
    npc::Kill,
    physics::{Layer, ALL_LAYERS},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_systems(Startup, setup_player)
            .add_systems(Update, shoot_npcs)
            .add_systems(
                PhysicsSchedule,
                move_player.before(PhysicsStepSet::BroadPhase),
            );
    }
}

#[derive(Component, Reflect)]
pub struct Player {
    speed: f32,
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
            Player { speed: 2. },
            PbrBundle {
                transform: Transform::from_xyz(0.0, player_height / 2., 0.0),
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

fn move_player(
    keyboard: Res<Input<KeyCode>>,
    debug_ui: Res<DebugUi>,
    mut q_player: Query<(&Transform, &Player, &mut LinearVelocity, &ShapeHits)>,
    mut ev_refocus: EventWriter<MainCameraFocusEvent>,
) {
    let Ok((player_tr, player, mut linear_velocity, ground_hits)) = q_player.get_single_mut()
    else {
        return;
    };

    if !ground_hits.is_empty() {
        linear_velocity.y = 0.0;
    } else {
        linear_velocity.y -= 0.4;
    }

    if !debug_ui.has_focus() {
        if keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up) {
            linear_velocity.z -= player.speed;
        }
        if keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left) {
            linear_velocity.x -= player.speed;
        }
        if keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down) {
            linear_velocity.z += player.speed;
        }
        if keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right) {
            linear_velocity.x += player.speed;
        }
        if keyboard.just_pressed(KeyCode::Space) && !ground_hits.is_empty() {
            linear_velocity.y += 20.0;
        }
    }

    linear_velocity.x *= 0.8;
    linear_velocity.y *= 0.98;
    linear_velocity.z *= 0.8;

    ev_refocus.send(MainCameraFocusEvent {
        focus: player_tr.translation,
    });
}

fn shoot_npcs(
    spatial_query: SpatialQuery,
    q_player: Query<&Transform, With<Player>>,
    mut cmd: Commands,
) {
    let Ok(tr) = q_player.get_single() else {
        return;
    };
    if let Some(projection) = spatial_query.project_point(
        tr.translation,
        true,
        SpatialQueryFilter::new().with_masks([Layer::NPC]),
    ) {
        let dist = (tr.translation - projection.point).length();
        if dist < 1. {
            let Some(mut npc) = cmd.get_entity(projection.entity) else {
                return;
            };
            npc.insert(Kill);
        }
    }
}
