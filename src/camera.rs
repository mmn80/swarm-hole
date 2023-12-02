use bevy::{
    core_pipeline::bloom::BloomSettings,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    transform::TransformSystem,
    window::PrimaryWindow,
};
use bevy_xpbd_3d::PhysicsSet;

use crate::app::is_running;

pub struct MainCameraPlugin;

impl Plugin for MainCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MainCamera>()
            .add_event::<MainCameraFocusEvent>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                main_camera
                    .after(PhysicsSet::Sync)
                    .before(TransformSystem::TransformPropagate)
                    .run_if(is_running),
            );
    }
}

#[derive(Component, Reflect)]
pub struct MainCamera {
    pub focus: Vec3,
    pub radius: f32,
    #[reflect(ignore)]
    pub mouse_ray: Option<Ray>,
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
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::Y, Vec3::Y),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        BloomSettings::NATURAL,
        MainCamera {
            radius,
            ..default()
        },
    ));
}

const TWO_PI: f32 = std::f32::consts::PI * 2.0;

/// zoom with scroll wheel, orbit with right mouse click
fn main_camera(
    mouse: Res<Input<MouseButton>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut ev_focus: EventReader<MainCameraFocusEvent>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(&mut MainCamera, &mut Transform, &GlobalTransform, &Camera)>,
) {
    let orbit_button = MouseButton::Right;

    let mut mouse_move = Vec2::ZERO;
    for ev in ev_motion.read() {
        if mouse.pressed(orbit_button) {
            mouse_move += ev.delta;
        }
    }

    let mut scroll = 0.0;
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }

    let (cursor_pos, window) = {
        let win = q_window.single();
        (win.cursor_position(), Vec2::new(win.width(), win.height()))
    };

    for (mut main_camera, mut camera_tr, camera_gtr, camera) in &mut q_camera {
        if let Some(pos) = cursor_pos {
            main_camera.mouse_ray = camera.viewport_to_world(camera_gtr, pos);
        }

        let mut any = false;
        if mouse_move.length_squared() > f32::EPSILON {
            any = true;
            let delta_x = { mouse_move.x / window.x * TWO_PI };
            let delta_y = mouse_move.y / window.y * TWO_PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            camera_tr.rotation = yaw * camera_tr.rotation; // rotate around global y axis
            camera_tr.rotation = camera_tr.rotation * pitch; // rotate around local x axis
        } else if scroll.abs() > 0.0 {
            any = true;
            main_camera.radius -= scroll * main_camera.radius * 0.2;
            main_camera.radius = f32::max(main_camera.radius, 0.05);
        }
        for focus_ev in ev_focus.read() {
            any = true;
            main_camera.focus = focus_ev.focus;
        }

        if any {
            let rot_matrix = Mat3::from_quat(camera_tr.rotation);
            camera_tr.translation =
                main_camera.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, main_camera.radius));
        }
    }
}
