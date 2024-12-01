use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

use crate::app::AppState;

pub struct MainLightsPlugin;

impl Plugin for MainLightsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 80.,
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(Startup, spawn_main_lights)
        .add_systems(
            Update,
            animate_light_direction.run_if(in_state(AppState::Run)),
        );
    }
}

fn spawn_main_lights(mut cmd: Commands) {
    cmd.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.1,
            maximum_distance: 200.0,
            first_cascade_far_bound: 5.0,
            overlap_proportion: 0.2,
        }
        .build(),
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
    ));
}

fn animate_light_direction(
    time: Res<Time>,
    mut q_light: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut tr in &mut q_light {
        tr.rotate(Quat::from_rotation_y(time.delta_secs() * 0.1));
    }
}
