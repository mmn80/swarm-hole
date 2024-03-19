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
    app::{AppState, MainMenuPlugin},
    camera::MainCameraPlugin,
    debug_ui::DebugUiPlugin,
    light::MainLightsPlugin,
    npc::NpcPlugin,
    physics::MainPhysicsPlugin,
    player::PlayerPlugin,
    skills::SkillPluginGroup,
    terrain::TerrainPlugin,
    ui::{MainUiPlugin, INFINITE_TEMP_COLOR},
    vfx::VfxPlugin,
};

fn main() {
    let mut wgpu_settings = WgpuSettings {
        // Remove this if you don't need it.
        // Leaving this as default gives errors on my AMD Radeon RX 5700 where wgpu thinks it's on Vulkan but Bevy thinks it's DX12.
        // Forcing Vulkan used to fix it before but I'm now getting perhaps unrelated fatal Vulkan validation errors and forcing DX12 seems to work.
        backends: Some(Backends::DX12),
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
                    synchronous_pipeline_compilation: false,
                }),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            HanabiPlugin,
        ))
        .init_state::<AppState>()
        .add_plugins((
            MainMenuPlugin,
            MainPhysicsPlugin,
            MainCameraPlugin,
            MainLightsPlugin,
            MainUiPlugin,
            TerrainPlugin,
            PlayerPlugin,
            NpcPlugin,
            SkillPluginGroup,
            VfxPlugin,
            DebugUiPlugin,
        ))
        .run();
}
