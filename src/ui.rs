use bevy::prelude::*;

use crate::{npc::Health, player::Player};

pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, update_ui);
    }
}

#[derive(Component)]
struct HpText;

#[derive(Component)]
struct XpText;

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

        parent.spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::Stretch,
                ..default()
            },
            ..default()
        });
    });
}

fn update_ui(
    q_player: Query<(&Player, &Health)>,
    mut q_hp: Query<&mut Text, (With<HpText>, Without<XpText>)>,
    mut q_xp: Query<&mut Text, (With<XpText>, Without<HpText>)>,
) {
    let Ok(mut txt_hp) = q_hp.get_single_mut() else {
        return;
    };
    let Ok(mut txt_xp) = q_xp.get_single_mut() else {
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
