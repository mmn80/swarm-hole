use avian3d::prelude::*;
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{
        RenderPlugin,
        settings::{Backends, WgpuFeatures, WgpuSettings},
    },
};
use bevy_hanabi::prelude::*;

use swarm_hole::{
    app::{AppState, InGame, MainMenuPlugin},
    camera::MainCameraPlugin,
    debug_ui::DebugUiPlugin,
    light::MainLightsPlugin,
    npc::NpcPlugin,
    physics::MainPhysicsPlugin,
    player::PlayerPlugin,
    skills::SkillPluginGroup,
    terrain::TerrainPlugin,
    ui::{INFINITE_TEMP_COLOR, MainUiPlugin},
    vfx::VfxPlugin,
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
                    ..default()
                }),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            HanabiPlugin,
        ))
        .init_state::<AppState>()
        .add_computed_state::<InGame>()
        .enable_state_scoped_entities::<AppState>()
        .enable_state_scoped_entities::<InGame>()
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
