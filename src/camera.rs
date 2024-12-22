use bevy::{
    core_pipeline::bloom::Bloom,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    transform::TransformSystem,
    window::PrimaryWindow,
};

use crate::app::InGame;

pub struct MainCameraPlugin;

impl Plugin for MainCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MainCamera>()
            .add_event::<MainCameraFocusEvent>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                PostUpdate,
                main_camera
                    .before(TransformSystem::TransformPropagate)
                    .run_if(in_state(InGame)),
            );
    }
}

#[derive(Component, Reflect)]
pub struct MainCamera {
    pub focus: Vec3,
    pub radius: f32,
    #[reflect(ignore)]
    pub mouse_ray: Option<Ray3d>,
}

const START_DIST: f32 = 75.0;

impl Default for MainCamera {
    fn default() -> Self {
        MainCamera {
            focus: Vec3::Y,
            radius: 5.0,
            mouse_ray: None,
        }
    }
}

#[derive(Event)]
pub struct MainCameraFocusEvent {
    pub focus: Vec3,
}

pub const UI_CAMERA_LAYER: u8 = 1;

#[derive(Component)]
pub struct UiCamera;

fn spawn_camera(mut cmd: Commands) {
    let translation = Vec3::new(0., START_DIST, START_DIST / 2.);
    let radius = translation.length();
    cmd.spawn((
        Camera3d::default(),
        Msaa::Sample4,
        Transform::from_translation(translation).looking_at(Vec3::Y, Vec3::Y),
        Camera {
            hdr: true,
            ..default()
        },
        Bloom::NATURAL,
        MainCamera {
            radius,
            ..default()
        },
    ));
}

const TWO_PI: f32 = std::f32::consts::PI * 2.0;

/// zoom with scroll wheel, orbit with right mouse click
fn main_camera(
    mouse: Res<ButtonInput<MouseButton>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut ev_focus: EventReader<MainCameraFocusEvent>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(&mut MainCamera, &mut Transform, &GlobalTransform, &Camera)>,
) {
    let scroll: f32 = ev_scroll.read().map(|ev| ev.y).sum();
    let new_focus = { ev_focus.read().last().map(|ev| ev.focus) };

    let mut delta = ev_motion.read().map(|ev| ev.delta).sum();
    if !mouse.pressed(MouseButton::Right) {
        delta = Vec2::ZERO;
    }
    let (cursor_pos, yaw, pitch) = {
        let win = q_window.single();
        (
            win.cursor_position(),
            Quat::from_rotation_y(-delta.x / win.width() * TWO_PI),
            Quat::from_rotation_x(-delta.y / win.height() * TWO_PI),
        )
    };

    for (mut main_camera, mut camera_tr, camera_gtr, camera) in &mut q_camera {
        if let Some(pos) = cursor_pos {
            main_camera.mouse_ray = camera.viewport_to_world(camera_gtr, pos).ok();
        }

        let mut any = false;
        if delta.length_squared() > f32::EPSILON {
            any = true;
            camera_tr.rotation = yaw * camera_tr.rotation;
            camera_tr.rotation = camera_tr.rotation * pitch;
        } else if scroll.abs() > 0.0 {
            any = true;
            main_camera.radius -= scroll * main_camera.radius * 0.2;
            main_camera.radius = f32::max(main_camera.radius, 0.05);
        }
        if let Some(new_focus) = new_focus {
            any = true;
            main_camera.focus = new_focus;
        }

        if any {
            let rot_matrix = Mat3::from_quat(camera_tr.rotation);
            camera_tr.translation =
                main_camera.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, main_camera.radius));
        }
    }
}
