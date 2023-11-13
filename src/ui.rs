use std::time::Duration;

use bevy::prelude::*;

use crate::{
    npc::{Health, Npc},
    player::Player,
};

pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (update_player_ui, update_time_ui, update_npcs_ui));
    }
}

#[derive(Resource, Default)]
pub struct GameState {
    pub started_time: Duration,
    pub ended_time: Duration,
}

fn setup_ui(mut cmd: Commands) {
    cmd.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    })
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
                    padding: UiRect::right(Val::Px(20.)),
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

        parent
            .spawn(NodeBundle {
                style: Style {
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::left(Val::Px(20.)),
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_sections([
                        TextSection::new(
                            "Time: ",
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
                    TimeText,
                ));
                parent.spawn((
                    TextBundle::from_sections([
                        TextSection::new(
                            "Npcs: ",
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
}

#[derive(Component)]
struct HpText;

#[derive(Component)]
struct XpText;

fn update_player_ui(
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
    txt_hp.sections[1].value = format!("{}", health.0 as u32);
    txt_xp.sections[1].value = format!("{}", player.xp);
}

#[derive(Component)]
struct TimeText;

fn update_time_ui(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut q_txt: Query<&mut Text, With<TimeText>>,
) {
    let Ok(mut txt_time) = q_txt.get_single_mut() else {
        return;
    };
    if game_state.started_time.is_zero() {
        txt_time.sections[1].value = "-".to_string();
    } else {
        let ended = if game_state.ended_time.is_zero() {
            time.elapsed()
        } else {
            game_state.ended_time
        };
        let all_sec = (ended - game_state.started_time).as_secs_f32();
        let min = (all_sec / 60.) as u32;
        let sec = all_sec as u32 - min * 60;
        txt_time.sections[1].value = format!("{min:02}:{sec:02}");
    }
}

#[derive(Component)]
struct NpcsText;

fn update_npcs_ui(q_npc: Query<With<Npc>>, mut q_txt: Query<&mut Text, With<NpcsText>>) {
    let Ok(mut txt_npcs) = q_txt.get_single_mut() else {
        return;
    };
    let npcs = q_npc.iter().count();
    if npcs == 0 {
        txt_npcs.sections[1].value = "-".to_string();
    } else {
        txt_npcs.sections[1].value = format!("{npcs:02}");
    }
}
