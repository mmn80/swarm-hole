use bevy::{
    app::AppExit,
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_xpbd_3d::prelude::*;

use swarm_hole::{
    camera::MainCameraPlugin,
    debug_ui::DebugUiPlugin,
    light::{MainLightsPlugin, INFINITE_TEMP_COLOR},
    materials::BasicMaterialsPlugin,
    physics::MainPhysicsPlugin,
    player::PlayerPlugin,
    terrain::TerrainPlugin,
};

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(INFINITE_TEMP_COLOR))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Swarm Hole".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..Default::default()
                    }),
                }),
        )
        .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        .add_plugins((
            DebugUiPlugin,
            BasicMaterialsPlugin,
            MainPhysicsPlugin,
            MainCameraPlugin,
            MainLightsPlugin,
            TerrainPlugin,
            PlayerPlugin,
        ))
        .add_systems(Update, exit_system)
        .run();
}

fn exit_system(keyboard: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard.just_released(KeyCode::Q) && keyboard.pressed(KeyCode::ControlLeft) {
        exit.send(AppExit);
    }
}
