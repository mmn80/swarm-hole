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
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InGame;

impl ComputedStates for InGame {
    type SourceStates = AppState;

    fn compute(sources: AppState) -> Option<InGame> {
        match sources {
            AppState::Run
            | AppState::Paused
            | AppState::Upgrade
            | AppState::Lost
            | AppState::Won => Some(InGame),
            _ => None,
        }
    }
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
                    exited: AppState::Menu,
                    entered: AppState::Run,
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
    let enter = keyboard.just_pressed(KeyCode::Enter) && !debug_ui.has_focus();
    let esc = keyboard.just_pressed(KeyCode::Escape) && !debug_ui.has_focus();
    match *app_state.get() {
        AppState::Lost | AppState::Won => {
            if enter || esc {
                next_state.set(AppState::Menu);
            }
        }
        AppState::Paused => {
            if enter {
                next_state.set(AppState::Run);
            } else if esc {
                next_state.set(AppState::Menu);
            }
        }
        AppState::Menu => {
            if esc {
                exit.send(AppExit::Success);
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

#[derive(Component)]
struct MainMenuUi;

fn setup_menu(mut cmd: Commands) {
    cmd.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        MainMenuUi,
    ))
    .with_children(|parent| {
        parent
            .spawn((
                Button,
                Node {
                    width: Val::Px(200.),
                    height: Val::Px(90.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_NORMAL_COLOR.into()),
                BorderRadius::all(Val::Px(20.0)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text("RUN".to_string()),
                    TextFont {
                        font_size: 60.0,
                        ..default()
                    },
                    TextColor(INFINITE_TEMP_COLOR),
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
