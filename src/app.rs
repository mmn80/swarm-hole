use std::time::Duration;

use bevy::{app::AppExit, prelude::*};

use crate::{
    debug_ui::DebugUi,
    ui::{BUTTON_HOVERED_COLOR, BUTTON_NORMAL_COLOR, BUTTON_PRESSED_COLOR, INFINITE_TEMP_COLOR},
};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    Run,
    Paused,
    Upgrade,
    Lost,
    Won,
    Cleanup,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunState>()
            .add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(OnExit(AppState::Menu), cleanup_menu)
            .add_systems(
                Update,
                (
                    update_app_state,
                    update_menu.run_if(in_state(AppState::Menu)),
                ),
            )
            .add_systems(
                OnTransition {
                    from: AppState::Menu,
                    to: AppState::Run,
                },
                start_run,
            );
    }
}

#[derive(Resource, Default)]
pub struct RunState {
    pub run_time: Duration,
    pub live_npcs: u32,
}

fn start_run(mut run_state: ResMut<RunState>) {
    run_state.run_time = Duration::ZERO;
    run_state.live_npcs = 0;
}

fn update_app_state(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut run_state: ResMut<RunState>,
    debug_ui: Res<DebugUi>,
    mut exit: EventWriter<AppExit>,
) {
    let enter = keyboard.just_pressed(KeyCode::Return) && !debug_ui.has_focus();
    let esc = keyboard.just_pressed(KeyCode::Escape) && !debug_ui.has_focus();
    match *app_state.get() {
        AppState::Cleanup => {
            next_state.set(AppState::Menu);
        }
        AppState::Lost | AppState::Won => {
            if enter || esc {
                next_state.set(AppState::Cleanup);
            }
        }
        AppState::Paused => {
            if enter {
                next_state.set(AppState::Run);
            } else if esc {
                next_state.set(AppState::Cleanup);
            }
        }
        AppState::Menu => {
            if esc {
                exit.send(AppExit);
            } else if enter {
                next_state.set(AppState::Run);
            }
        }
        AppState::Upgrade => {
            if esc {
                next_state.set(AppState::Paused);
            }
        }
        AppState::Run => {
            if esc || enter {
                next_state.set(AppState::Paused);
            } else {
                run_state.run_time += time.delta();
            }
        }
    }
}

pub fn is_running(app_state: Res<State<AppState>>) -> bool {
    let state = *app_state.get();
    return state == AppState::Run
        || state == AppState::Paused
        || state == AppState::Upgrade
        || state == AppState::Lost
        || state == AppState::Won;
}

#[derive(Component)]
struct MainMenuUi;

fn setup_menu(mut cmd: Commands) {
    cmd.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        },
        MainMenuUi,
    ))
    .with_children(|parent| {
        parent
            .spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(200.),
                    height: Val::Px(90.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BUTTON_NORMAL_COLOR.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "RUN",
                    TextStyle {
                        font_size: 60.0,
                        color: INFINITE_TEMP_COLOR,
                        ..default()
                    },
                ));
            });
    });
}

fn update_menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut q_interaction: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut q_interaction {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED_COLOR.into();
                next_state.set(AppState::Run);
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

fn cleanup_menu(mut cmd: Commands, q_main_menu: Query<Entity, With<MainMenuUi>>) {
    for entity in &q_main_menu {
        cmd.entity(entity).despawn_recursive();
    }
}
