use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use rand::prelude::*;

use crate::{
    app::{is_running, AppState, RunState},
    player::Player,
    skills::{health::Health, xp_drops::XpGatherState},
};

pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnTransition {
                from: AppState::Menu,
                to: AppState::Run,
            },
            (
                setup_fps_ui,
                setup_top_bar_ui,
                setup_upgrade_ui,
                setup_paused_ui,
            ),
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
                update_skill_upgrade_ui,
            )
                .run_if(is_running),
        );
    }
}

pub const INFINITE_TEMP_COLOR: Color = Color::rgb_linear(
    148. / u8::MAX as f32,
    177. / u8::MAX as f32,
    255. / u8::MAX as f32,
);
pub const BUTTON_NORMAL_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);
pub const BUTTON_HOVERED_COLOR: Color = Color::rgb(0.25, 0.25, 0.25);
pub const BUTTON_PRESSED_COLOR: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct MainUi;

// top bar

fn setup_top_bar_ui(mut cmd: Commands) {
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
                            "HP:  ",
                            TextStyle {
                                font_size: 30.0,
                                ..default()
                            },
                        ),
                        TextSection::new(
                            "-",
                            TextStyle {
                                font_size: 30.0,
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
                            "NPC: ",
                            TextStyle {
                                font_size: 30.0,
                                ..default()
                            },
                        ),
                        TextSection::new(
                            "-",
                            TextStyle {
                                font_size: 30.0,
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

        parent.spawn((
            TextBundle::from_section(
                "00:00",
                TextStyle {
                    font_size: 60.0,
                    ..default()
                },
            )
            .with_style(Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::horizontal(Val::Px(60.)),
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
                            "LVL: ",
                            TextStyle {
                                font_size: 30.0,
                                ..default()
                            },
                        ),
                        TextSection::new(
                            "-",
                            TextStyle {
                                font_size: 30.0,
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
                    LevelText,
                ));

                parent.spawn((
                    TextBundle::from_sections([
                        TextSection::new(
                            "XP:  ",
                            TextStyle {
                                font_size: 30.0,
                                ..default()
                            },
                        ),
                        TextSection::new(
                            "-",
                            TextStyle {
                                font_size: 30.0,
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

        parent.spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::Stretch,
                ..default()
            },
            ..default()
        });
    });
}

#[derive(Component)]
struct HpText;

#[derive(Component)]
struct XpText;

#[derive(Component)]
struct LevelText;

fn update_player_ui(
    time: Res<Time>,
    q_player: Query<(&XpGatherState, &Health), With<Player>>,
    mut q_hp_txt: Query<&mut Text, (With<HpText>, Without<XpText>)>,
    mut q_xp_txt: Query<&mut Text, (With<XpText>, Without<HpText>)>,
    mut q_level_txt: Query<&mut Text, (With<LevelText>, Without<HpText>, Without<XpText>)>,
) {
    let Ok(mut txt_hp) = q_hp_txt.get_single_mut() else {
        return;
    };
    let Ok(mut txt_xp) = q_xp_txt.get_single_mut() else {
        return;
    };
    let Ok(mut txt_level) = q_level_txt.get_single_mut() else {
        return;
    };
    let Ok((xp_gather_state, health)) = q_player.get_single() else {
        txt_hp.sections[1].value = "-".to_string();
        txt_xp.sections[1].value = "-".to_string();
        return;
    };
    txt_hp.sections[1].style.color = if health.hp < 50. {
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
    {
        let (hp, max_hp) = (health.hp as u32, health.max_hp as u32);
        if hp < max_hp {
            txt_hp.sections[1].value = format!("{}/{}", hp, max_hp);
        } else {
            txt_hp.sections[1].value = format!("{}", hp);
        }
    }
    txt_xp.sections[1].value = format!("{}", xp_gather_state.xp);
    txt_level.sections[1].value = format!("{}", xp_gather_state.level);
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

// FPS

fn setup_fps_ui(mut cmd: Commands) {
    cmd.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 40.0,
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

// paused & game over panel

fn setup_paused_ui(mut cmd: Commands) {
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
        parent
            .spawn((NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BackgroundColor::from(Color::rgba(0.15, 0.15, 0.15, 0.8)),
                ..default()
            },))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "PAUSED",
                        TextStyle {
                            font_size: 100.0,
                            color: INFINITE_TEMP_COLOR,
                            ..default()
                        },
                    )
                    .with_text_alignment(TextAlignment::Center)
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(50.)),
                        ..default()
                    }),
                    AppStateText,
                ));
                parent.spawn((TextBundle::from_section(
                    "press ENTER to continue",
                    TextStyle {
                        font_size: 30.0,
                        ..default()
                    },
                )
                .with_text_alignment(TextAlignment::Center)
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.)),
                    ..default()
                }),));
            });
    });
}

#[derive(Component)]
struct AppStateRoot;

#[derive(Component)]
struct AppStateText;

const LOSE_STRS: [&str; 10] = [
    "DEAD",
    "DECEASED",
    "GONE",
    "DEPARTED",
    "TERMINATED",
    "FINISHED",
    "FALLEN",
    "EXTINCT",
    "BLUNTED",
    "KAPUT",
];

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
    } else if style.display == Display::None {
        style.display = Display::Flex;
        if state == AppState::Paused {
            txt_run_state.sections[0].value = "PAUSED".to_string();
            txt_run_state.sections[0].style.color = INFINITE_TEMP_COLOR;
        } else if state == AppState::Won {
            txt_run_state.sections[0].value = "DONE".to_string();
            txt_run_state.sections[0].style.color = Color::GOLD;
        } else if state == AppState::Lost {
            txt_run_state.sections[0].value = LOSE_STRS[thread_rng().gen_range(0..10)].to_string();
            txt_run_state.sections[0].style.color = Color::ORANGE_RED;
        }
    }
}

// player skill upgrade menu

fn setup_upgrade_ui(mut cmd: Commands) {
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
        SkillUpgradeRoot,
        MainUi,
    ))
    .with_children(|parent| {
        add_skill_upgrade_button(parent, 0);
        add_skill_upgrade_button(parent, 1);
        add_skill_upgrade_button(parent, 2);
    });
}

fn add_skill_upgrade_button(parent: &mut ChildBuilder<'_, '_, '_>, index: u8) {
    if index > 0 {
        parent.spawn(NodeBundle {
            style: Style {
                height: Val::Px(50.),
                ..default()
            },
            ..default()
        });
    }

    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    min_width: Val::Px(600.),
                    ..default()
                },
                background_color: BUTTON_NORMAL_COLOR.into(),
                ..default()
            },
            SkillUpgradeButton(index),
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    format!("Upgrade {index}"),
                    TextStyle {
                        font_size: 30.0,
                        color: INFINITE_TEMP_COLOR,
                        ..default()
                    },
                )
                .with_text_alignment(TextAlignment::Center)
                .with_style(Style {
                    margin: UiRect::all(Val::Px(40.)),
                    ..default()
                }),
                SkillUpgradeText(index),
            ));
        });
}

#[derive(Component)]
struct SkillUpgradeRoot;

#[derive(Component)]
struct SkillUpgradeButton(u8);

#[derive(Component)]
struct SkillUpgradeText(u8);

fn update_skill_upgrade_ui(
    mut q_interaction: Query<
        (&Interaction, &mut BackgroundColor, &SkillUpgradeButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, SkillUpgradeButton(_idx)) in &mut q_interaction {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED_COLOR.into();
            }
            Interaction::Hovered => {
                *color = BUTTON_HOVERED_COLOR.into();
            }
            Interaction::None => {
                *color = BUTTON_NORMAL_COLOR.into();
            }
        }
    }
}

// cleanup main UI

fn cleanup_ui(q_main_ui: Query<Entity, With<MainUi>>, mut cmd: Commands) {
    for entity in &q_main_ui {
        cmd.entity(entity).despawn_recursive();
    }
}
