use bevy::{
    color::palettes::css::{AQUAMARINE, GOLD, ORANGE_RED, YELLOW},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use rand::prelude::*;

use crate::{
    app::{AppState, InGame, RunState},
    player::Player,
    skills::{
        health::{Health, MaxHealth},
        xp::XpGatherState,
        SkillUpgradeOptions, Skills,
    },
};

pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnTransition {
                exited: AppState::Menu,
                entered: AppState::Run,
            },
            (
                setup_fps_ui,
                setup_top_bar_ui,
                setup_upgrade_ui.before(setup_paused_ui),
                setup_paused_ui,
            ),
        )
        .add_systems(
            Update,
            (
                update_fps,
                update_player_ui,
                update_run_time_ui,
                update_app_state_ui,
                update_npcs_ui,
            )
                .run_if(in_state(InGame)),
        )
        .add_systems(OnEnter(AppState::Upgrade), init_skill_upgrade_ui)
        .add_systems(
            Update,
            update_skill_upgrade_ui.run_if(in_state(AppState::Upgrade)),
        );
    }
}

pub const INFINITE_TEMP_COLOR: Color = Color::linear_rgb(
    148. / u8::MAX as f32,
    177. / u8::MAX as f32,
    255. / u8::MAX as f32,
);
pub const BUTTON_NORMAL_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
pub const BUTTON_HOVERED_COLOR: Color = Color::srgb(0.25, 0.25, 0.25);
pub const BUTTON_PRESSED_COLOR: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct MainUi;

// top bar

fn setup_top_bar_ui(mut cmd: Commands) {
    cmd.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        MainUi,
        StateScoped(InGame),
    ))
    .with_children(|parent| {
        parent.spawn(Node {
            align_items: AlignItems::Stretch,
            ..default()
        });

        parent
            .spawn(Node {
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn((
                        Text::new("HP:  "),
                        Node {
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        TextLayout {
                            justify: JustifyText::Left,
                            ..default()
                        },
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        HpText,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextSpan::new("-"),
                            TextFont {
                                font_size: 30.0,
                                ..default()
                            },
                        ));
                    });

                parent
                    .spawn((
                        Text::new("NPC: "),
                        Node {
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        TextLayout {
                            justify: JustifyText::Left,
                            ..default()
                        },
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        NpcsText,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextSpan::new("-"),
                            TextFont {
                                font_size: 30.0,
                                ..default()
                            },
                        ));
                    });
            });

        parent.spawn((
            Text::new("00:00"),
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::horizontal(Val::Px(60.)),
                ..default()
            },
            TextFont {
                font_size: 60.0,
                ..default()
            },
            TimeText,
        ));

        parent
            .spawn(Node {
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn((
                        Text::new("LVL: "),
                        Node {
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        TextLayout {
                            justify: JustifyText::Left,
                            ..default()
                        },
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        LevelText,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextSpan::new("-"),
                            TextFont {
                                font_size: 30.0,
                                ..default()
                            },
                        ));
                    });

                parent
                    .spawn((
                        Text::new("XP:  "),
                        Node {
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        TextLayout {
                            justify: JustifyText::Left,
                            ..default()
                        },
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        XpText,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextSpan::new("-"),
                            TextFont {
                                font_size: 30.0,
                                ..default()
                            },
                        ));
                    });
            });

        parent.spawn(Node {
            align_items: AlignItems::Stretch,
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
    q_player: Query<(&XpGatherState, &Health, &MaxHealth), With<Player>>,
    q_hp_txt: Query<Entity, With<HpText>>,
    q_xp_txt: Query<Entity, With<XpText>>,
    q_level_txt: Query<Entity, With<LevelText>>,
    mut writer: TextUiWriter,
) {
    let Ok(txt_hp) = q_hp_txt.get_single() else {
        return;
    };
    let Ok(txt_xp) = q_xp_txt.get_single() else {
        return;
    };
    let Ok(txt_level) = q_level_txt.get_single() else {
        return;
    };
    let Ok((xp_gather_state, health, max_health)) = q_player.get_single() else {
        *writer.text(txt_hp, 1) = "-".to_string();
        *writer.text(txt_xp, 1) = "-".to_string();
        return;
    };
    *writer.color(txt_hp, 1) = TextColor(if health.0 < 50. {
        let sec = time.elapsed_secs();
        Color::Srgba(Srgba {
            red: (4. * sec).sin() / 4.0 + 1.0,
            green: 0.25,
            blue: 0.,
            alpha: 1.0,
        })
    } else {
        Color::default()
    });
    {
        let (hp, max_hp) = (health.0 as u32, max_health.max_hp);
        if hp < max_hp {
            *writer.text(txt_hp, 1) = format!("{}/{}", hp, max_hp);
        } else {
            *writer.text(txt_hp, 1) = format!("{}", hp);
        }
    }
    *writer.text(txt_xp, 1) = format!("{}", xp_gather_state.xp);
    *writer.text(txt_level, 1) = format!("{}", xp_gather_state.get_gather_level());
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
        txt_time.0 = format!("{min:02}:{sec:02}");
    } else {
        txt_time.0 = "00:00".to_string();
    }
}

#[derive(Component)]
struct NpcsText;

fn update_npcs_ui(
    run_state: Res<RunState>,
    q_txt: Query<Entity, With<NpcsText>>,
    mut writer: TextUiWriter,
) {
    let Ok(txt_npcs) = q_txt.get_single() else {
        return;
    };
    if run_state.live_npcs == 0 {
        *writer.text(txt_npcs, 1) = "-".to_string();
    } else {
        *writer.text(txt_npcs, 1) = format!("{:02}", run_state.live_npcs);
    }
}

// FPS

fn setup_fps_ui(mut cmd: Commands) {
    cmd.spawn((
        Text::default(),
        Node {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(YELLOW.into()),
        FpsText,
        MainUi,
        StateScoped(InGame),
    ));
}

#[derive(Component)]
struct FpsText;

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut q_fps_txt: Query<&mut Text, With<FpsText>>) {
    let Ok(mut txt_fps) = q_fps_txt.get_single_mut() else {
        return;
    };
    let mut fps = 0.0;
    if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
            fps = fps_smoothed;
        }
    }
    txt_fps.0 = format!("{fps:.0}");
}

// paused & game over panel

fn setup_paused_ui(mut cmd: Commands) {
    cmd.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        AppStateRoot,
        MainUi,
        StateScoped(InGame),
    ))
    .with_children(|parent| {
        parent
            .spawn((
                Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
                BorderRadius::all(Val::Px(50.0)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("PAUSED"),
                    Node {
                        margin: UiRect::all(Val::Px(50.)),
                        ..default()
                    },
                    TextFont {
                        font_size: 100.0,
                        ..default()
                    },
                    TextColor(INFINITE_TEMP_COLOR),
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    AppStateText(AppState::Menu),
                ));
                parent.spawn((
                    Text::new("press ENTER to continue"),
                    Node {
                        margin: UiRect::all(Val::Px(50.)),
                        ..default()
                    },
                    TextFont {
                        font_size: 30.0,
                        ..default()
                    },
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                ));
            });
    });
}

#[derive(Component)]
struct AppStateRoot;

#[derive(Component)]
struct AppStateText(AppState);

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
    mut q_app_state_root: Query<&mut Node, With<AppStateRoot>>,
    mut q_txt_app_state: Query<(&mut Text, &mut TextColor, &mut AppStateText)>,
) {
    let Ok(mut node) = q_app_state_root.get_single_mut() else {
        return;
    };
    let Ok((mut txt_run_state, mut txt_run_state_color, mut marker)) =
        q_txt_app_state.get_single_mut()
    else {
        return;
    };
    let state = *app_state.get();
    match state {
        AppState::Paused => {
            if marker.0 != AppState::Paused {
                marker.0 = AppState::Paused;
                node.display = Display::Flex;
                txt_run_state.0 = "PAUSED".to_string();
                txt_run_state_color.0 = INFINITE_TEMP_COLOR;
            }
        }
        AppState::Lost => {
            if marker.0 != AppState::Lost {
                marker.0 = AppState::Lost;
                node.display = Display::Flex;
                txt_run_state.0 = LOSE_STRS[thread_rng().gen_range(0..10)].to_string();
                txt_run_state_color.0 = ORANGE_RED.into();
            }
        }
        AppState::Won => {
            if marker.0 != AppState::Won {
                marker.0 = AppState::Won;
                node.display = Display::Flex;
                txt_run_state.0 = "DONE".to_string();
                txt_run_state_color.0 = GOLD.into();
            }
        }
        other => {
            marker.0 = other;
            node.display = Display::None;
        }
    }
}

// player skill upgrade menu

fn setup_upgrade_ui(mut cmd: Commands) {
    cmd.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        SkillUpgradeRoot,
        MainUi,
        StateScoped(InGame),
    ))
    .with_children(|parent| {
        add_skill_upgrade_button(parent, 0);
        add_skill_upgrade_button(parent, 1);
        add_skill_upgrade_button(parent, 2);
    });
}

fn add_skill_upgrade_button(parent: &mut ChildBuilder<'_>, index: usize) {
    if index > 0 {
        parent.spawn(Node {
            height: Val::Px(50.),
            ..default()
        });
    }

    parent
        .spawn((
            Button,
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                min_width: Val::Px(600.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(BUTTON_NORMAL_COLOR),
            BorderRadius::all(Val::Px(20.0)),
            SkillUpgradeButton(index),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Text::new(format!("Upgrade {index}")),
                    Node {
                        margin: UiRect::new(Val::Px(40.), Val::Px(40.), Val::Px(40.), Val::Px(10.)),
                        ..default()
                    },
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                    TextFont {
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(AQUAMARINE.into()),
                    SkillUpgradeText(index),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextSpan::default(),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(INFINITE_TEMP_COLOR),
                    ));
                });

            parent.spawn((
                Text(format!("upgrade details for {index}")),
                Node {
                    margin: UiRect::new(Val::Px(40.), Val::Px(40.), Val::Px(10.), Val::Px(40.)),
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(INFINITE_TEMP_COLOR),
                SkillUpgradeDetailsText(index),
            ));
        });
}

#[derive(Component)]
struct SkillUpgradeRoot;

#[derive(Component)]
struct SkillUpgradeButton(usize);

#[derive(Component)]
struct SkillUpgradeText(usize);

#[derive(Component)]
struct SkillUpgradeDetailsText(usize);

fn init_skill_upgrade_ui(
    upgrade_options: Res<SkillUpgradeOptions>,
    skills: Res<Skills>,
    mut q_root: Query<&mut Node, With<SkillUpgradeRoot>>,
    mut q_buttons: Query<(&mut Node, &SkillUpgradeButton), Without<SkillUpgradeRoot>>,
    q_texts: Query<(Entity, &SkillUpgradeText)>,
    q_detail_texts: Query<(Entity, &SkillUpgradeDetailsText), Without<SkillUpgradeText>>,
    mut writer: TextUiWriter,
) {
    for (text, marker) in &q_texts {
        if let Some((skill, level)) = upgrade_options.skills.get(marker.0) {
            if let Some(skill_name) = skills.skills.get(skill) {
                *writer.text(text, 0) = skill_name.to_string();
                *writer.text(text, 1) = format!(" Level {level}");
            }
        }
    }
    for (text, marker) in &q_detail_texts {
        if let Some((skill, level)) = upgrade_options.skills.get(marker.0) {
            if let Some(levels) = skills.upgrades.get(skill) {
                let mut str = String::new();
                let spec = level.index(&levels).unwrap();
                for (attr, val) in spec {
                    if let Some(attr_meta) = skills.attributes.get(attr) {
                        if let Some(prev_level) = level.prev() {
                            if let Some(prev_spec) = prev_level.index(levels) {
                                if let Some(val_prev) = prev_spec.get(attr) {
                                    if let Some(delta) = val.delta(*val_prev) {
                                        if !delta.is_zero() {
                                            if !str.is_empty() {
                                                str.push_str(", ");
                                            }
                                            str.push_str(&format!(
                                                "{}: {delta}",
                                                attr_meta.ui_name
                                            ));
                                        }
                                    }
                                }
                            }
                        } else {
                            if !str.is_empty() {
                                str.push_str(", ");
                            }
                            str.push_str(&format!("{}: {val}", attr_meta.ui_name));
                        }
                    }
                }
                *writer.text(text, 0) = format!("{str}");
            }
        }
    }
    for (mut node, button) in &mut q_buttons {
        if button.0 >= upgrade_options.skills.len() {
            node.display = Display::None;
        } else {
            node.display = Display::Flex;
        }
    }
    for mut node in &mut q_root {
        node.display = Display::Flex;
    }
}

fn update_skill_upgrade_ui(
    mut upgrade_options: ResMut<SkillUpgradeOptions>,
    mut q_root: Query<&mut Node, With<SkillUpgradeRoot>>,
    mut q_interaction: Query<
        (&Interaction, &mut BackgroundColor, &SkillUpgradeButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, SkillUpgradeButton(idx)) in &mut q_interaction {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED_COLOR.into();
                upgrade_options.selected = upgrade_options.skills.get(*idx).copied();
                for mut node in &mut q_root {
                    node.display = Display::None;
                }
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
