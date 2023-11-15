use bevy::{
    core_pipeline::bloom::BloomSettings,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use crate::app::is_running;

pub struct MainCameraPlugin;

impl Plugin for MainCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MainCamera>()
            .add_event::<MainCameraFocusEvent>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, main_camera.run_if(is_running));
    }
}

#[derive(Component, Reflect)]
pub struct MainCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    #[reflect(ignore)]
    pub mouse_ray: Option<Ray>,
}

const START_DIST: f32 = 50.0;

impl Default for MainCamera {
    fn default() -> Self {
        MainCamera {
            focus: Vec3::Y,
            radius: 5.0,
            upside_down: false,
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

/// zoom with scroll wheel, orbit with right mouse click
fn main_camera(
    mouse: Res<Input<MouseButton>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut ev_cursor: EventReader<CursorMoved>,
    mut ev_focus: EventReader<MainCameraFocusEvent>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(&mut MainCamera, &mut Transform, &GlobalTransform, &Camera)>,
) {
    let orbit_button = MouseButton::Right;

    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if mouse.pressed(orbit_button) {
        for ev in ev_motion.read() {
            rotation_move += ev.delta;
        }
    }
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }
    if mouse.just_released(orbit_button) || mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    let cursor_pos = ev_cursor.read().last().map(|p| p.position);

    for (mut main_camera, mut camera_tr, camera_gtr, camera) in &mut q_camera {
        if let Some(pos) = cursor_pos {
            main_camera.mouse_ray = camera.viewport_to_world(camera_gtr, pos);
        }

        if orbit_button_changed {
            let up = camera_tr.rotation * Vec3::Y;
            main_camera.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = {
                let window = q_window.single();
                Vec2::new(window.width(), window.height())
            };
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if main_camera.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
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
