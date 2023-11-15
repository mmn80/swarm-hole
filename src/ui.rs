use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{
    app::{is_running, AppState, RunState, INFINITE_TEMP_COLOR},
    npc::Health,
    player::Player,
};

pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnTransition {
                from: AppState::Menu,
                to: AppState::Run,
            },
            setup_ui,
        )
        .add_systems(OnEnter(AppState::Cleanup), cleanup_ui)
        .add_systems(
            Update,
            (
                update_fps,
                update_player_ui,
                update_run_time_ui,
                update_app_state_ui,
                update_npcs_ui,
            )
                .run_if(is_running),
        );
    }
}

#[derive(Component)]
struct MainUi;

fn setup_ui(mut cmd: Commands) {
    cmd.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 20.0,
                color: Color::YELLOW,
                ..default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        }),
        FpsText,
        MainUi,
    ));

    cmd.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
        MainUi,
    ))
    .with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::Stretch,
                ..default()
            },
            ..default()
        });

        parent
            .spawn(NodeBundle {
                style: Style {
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_sections([
                        TextSection::new(
                            "HP: ",
                            TextStyle {
                                font_size: 20.0,
                                ..default()
                            },
                        ),
                        TextSection::new(
                            "-",
                            TextStyle {
                                font_size: 20.0,
                                ..default()
                            },
                        ),
                    ])
                    .with_text_alignment(TextAlignment::Left)
                    .with_style(Style {
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    }),
                    HpText,
                ));
                parent.spawn((
                    TextBundle::from_sections([
                        TextSection::new(
                            "XP: ",
                            TextStyle {
                                font_size: 20.0,
                                ..default()
                            },
                        ),
                        TextSection::new(
                            "-",
                            TextStyle {
                                font_size: 20.0,
                                ..default()
                            },
                        ),
                    ])
                    .with_text_alignment(TextAlignment::Left)
                    .with_style(Style {
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    }),
                    XpText,
                ));
            });

        parent.spawn((
            TextBundle::from_section(
                "00:00",
                TextStyle {
                    font_size: 40.0,
                    ..default()
                },
            )
            .with_style(Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::horizontal(Val::Px(40.)),
                ..default()
            }),
            TimeText,
        ));

        parent
            .spawn(NodeBundle {
                style: Style {
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_sections([
                        TextSection::new(
                            "NPC: ",
                            TextStyle {
                                font_size: 20.0,
                                ..default()
                            },
                        ),
                        TextSection::new(
                            "-",
                            TextStyle {
                                font_size: 20.0,
                                ..default()
                            },
                        ),
                    ])
                    .with_text_alignment(TextAlignment::Left)
                    .with_style(Style {
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    }),
                    NpcsText,
                ));
            });

        parent.spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::Stretch,
                ..default()
            },
            ..default()
        });
    });

    cmd.spawn((
        NodeBundle {
            style: Style {
                display: Display::None,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        },
        AppStateRoot,
        MainUi,
    ))
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "PAUSED",
                TextStyle {
                    font_size: 80.0,
                    color: INFINITE_TEMP_COLOR,
                    ..default()
                },
            )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                margin: UiRect::all(Val::Px(40.)),
                ..default()
            }),
            AppStateText,
        ));
        parent.spawn((TextBundle::from_section(
            "press ENTER to continue",
            TextStyle {
                font_size: 25.0,
                ..default()
            },
        )
        .with_text_alignment(TextAlignment::Center),));
    });
}

#[derive(Component)]
struct FpsText;

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut q_fps_txt: Query<&mut Text, With<FpsText>>) {
    let Ok(mut txt_fps) = q_fps_txt.get_single_mut() else {
        return;
    };
    let mut fps = 0.0;
    if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
            fps = fps_smoothed;
        }
    }
    txt_fps.sections[0].value = format!("{fps:.0}");
}

#[derive(Component)]
struct HpText;

#[derive(Component)]
struct XpText;

fn update_player_ui(
    time: Res<Time>,
    q_player: Query<(&Player, &Health)>,
    mut q_hp_txt: Query<&mut Text, (With<HpText>, Without<XpText>)>,
    mut q_xp_txt: Query<&mut Text, (With<XpText>, Without<HpText>)>,
) {
    let Ok(mut txt_hp) = q_hp_txt.get_single_mut() else {
        return;
    };
    let Ok(mut txt_xp) = q_xp_txt.get_single_mut() else {
        return;
    };
    let Ok((player, health)) = q_player.get_single() else {
        txt_hp.sections[1].value = "-".to_string();
        txt_xp.sections[1].value = "-".to_string();
        return;
    };
    txt_hp.sections[1].style.color = if health.0 < 50. {
        let sec = time.elapsed_seconds();
        Color::Rgba {
            red: (4. * sec).sin() / 4.0 + 1.0,
            green: 0.25,
            blue: 0.,
            alpha: 1.0,
        }
    } else {
        Color::default()
    };
    txt_hp.sections[1].value = format!("{}", health.0 as u32);
    txt_xp.sections[1].value = format!("{}", player.xp);
}

#[derive(Component)]
struct TimeText;

fn update_run_time_ui(run_state: Res<RunState>, mut q_txt_time: Query<&mut Text, With<TimeText>>) {
    let Ok(mut txt_time) = q_txt_time.get_single_mut() else {
        return;
    };
    if !run_state.run_time.is_zero() {
        let all_sec = run_state.run_time.as_secs_f32();
        let min = (all_sec / 60.) as u32;
        let sec = all_sec as u32 - min * 60;
        txt_time.sections[0].value = format!("{min:02}:{sec:02}");
    } else {
        txt_time.sections[0].value = "00:00".to_string();
    }
}
#[derive(Component)]
struct AppStateRoot;

#[derive(Component)]
struct AppStateText;

fn update_app_state_ui(
    app_state: Res<State<AppState>>,
    mut q_run_state_root: Query<&mut Style, With<AppStateRoot>>,
    mut q_txt_run_state: Query<&mut Text, With<AppStateText>>,
) {
    let Ok(mut style) = q_run_state_root.get_single_mut() else {
        return;
    };
    let Ok(mut txt_run_state) = q_txt_run_state.get_single_mut() else {
        return;
    };
    let state = *app_state.get();
    if state == AppState::Run {
        style.display = Display::None;
    } else {
        style.display = Display::Flex;
        if state == AppState::Paused {
            txt_run_state.sections[0].value = "PAUSED".to_string();
            txt_run_state.sections[0].style.color = INFINITE_TEMP_COLOR;
        } else if state == AppState::Won {
            txt_run_state.sections[0].value = "DONE".to_string();
            txt_run_state.sections[0].style.color = Color::GOLD;
        } else if state == AppState::Lost {
            txt_run_state.sections[0].value = "DONE FOR".to_string();
            txt_run_state.sections[0].style.color = Color::ORANGE_RED;
        }
    }
}

#[derive(Component)]
struct NpcsText;

fn update_npcs_ui(run_state: Res<RunState>, mut q_txt: Query<&mut Text, With<NpcsText>>) {
    let Ok(mut txt_npcs) = q_txt.get_single_mut() else {
        return;
    };
    if run_state.live_npcs == 0 {
        txt_npcs.sections[1].value = "-".to_string();
    } else {
        txt_npcs.sections[1].value = format!("{:02}", run_state.live_npcs);
    }
}

fn cleanup_ui(q_main_ui: Query<Entity, With<MainUi>>, mut cmd: Commands) {
    for entity in &q_main_ui {
        cmd.entity(entity).despawn_recursive();
    }
}
