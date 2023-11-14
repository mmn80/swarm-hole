use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{
        settings::{Backends, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_hanabi::prelude::*;
use bevy_xpbd_3d::prelude::*;

use swarm_hole::{
    camera::MainCameraPlugin,
    debug_ui::DebugUiPlugin,
    light::{MainLightsPlugin, INFINITE_TEMP_COLOR},
    materials::BasicMaterialsPlugin,
    npc::NpcPlugin,
    physics::MainPhysicsPlugin,
    player::PlayerPlugin,
    terrain::TerrainPlugin,
    ui::MainUiPlugin,
    vfx::VfxPlugin,
    weapons::WeaponPlugins,
};

fn main() {
    let mut wgpu_settings = WgpuSettings {
        backends: Some(Backends::VULKAN),
        ..Default::default()
    };
    wgpu_settings
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(INFINITE_TEMP_COLOR))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Swarm Hole".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: wgpu_settings.into(),
                }),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            HanabiPlugin,
        ))
        .add_plugins((
            DebugUiPlugin,
            BasicMaterialsPlugin,
            MainPhysicsPlugin,
            MainCameraPlugin,
            MainLightsPlugin,
            MainUiPlugin,
            TerrainPlugin,
            PlayerPlugin,
            NpcPlugin,
            WeaponPlugins,
            VfxPlugin,
        ))
        .run();
}
