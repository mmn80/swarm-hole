use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{camera::MainCameraFocusEvent, materials::BasicMaterials};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_systems(Startup, setup_player)
            .add_systems(Update, move_player);
    }
}

#[derive(Component, Reflect)]
pub struct Player;

fn setup_player(
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<BasicMaterials>,
    mut cmd: Commands,
) {
    let player_height = 2.;
    let player_width = 0.3;

    let id = cmd
        .spawn((
            Player,
            PbrBundle {
                transform: Transform::from_xyz(0.0, player_height / 2., 0.0),
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: player_width,
                    rings: 0,
                    depth: player_height - 2. * player_width,
                    latitudes: 16,
                    longitudes: 32,
                    uv_profile: shape::CapsuleUvProfile::Aspect,
                })),
                material: materials.player.clone(),
                ..default()
            },
            RigidBody::Kinematic,
            Collider::capsule(player_height - 2. * player_width, player_width),
        ))
        .id();
    cmd.entity(id).insert(Name::new(format!("Player ({id:?})")));
}

fn move_player(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut q_player: Query<&mut Transform, With<Player>>,
    mut ev_refocus: EventWriter<MainCameraFocusEvent>,
) {
    if !keyboard.any_pressed([KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D]) {
        return;
    }
    let Ok(mut player_tr) = q_player.get_single_mut() else {
        return;
    };

    let mut ds = time.delta_seconds() * 10.;
    if keyboard.pressed(KeyCode::ShiftLeft) {
        ds *= 4.;
    }
    let forward = -Vec3::Z;
    let right = Vec3::X;
    if keyboard.pressed(KeyCode::W) {
        player_tr.translation += ds * forward;
    } else if keyboard.pressed(KeyCode::S) {
        player_tr.translation -= ds * forward;
    }
    if keyboard.pressed(KeyCode::A) {
        player_tr.translation -= ds * right;
    } else if keyboard.pressed(KeyCode::D) {
        player_tr.translation += ds * right;
    }

    ev_refocus.send(MainCameraFocusEvent {
        focus: player_tr.translation,
    });
}
