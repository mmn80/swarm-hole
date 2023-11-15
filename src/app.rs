use std::time::Duration;

use bevy::{app::AppExit, prelude::*};

use crate::debug_ui::DebugUi;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    Run,
    Paused,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunState>()
            .add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(
                Update,
                (update_menu, menu_key_shortcuts).run_if(in_state(AppState::Menu)),
            )
            .add_systems(OnExit(AppState::Menu), (cleanup_menu, start_run))
            .add_systems(
                OnTransition {
                    from: AppState::Menu,
                    to: AppState::Run,
                },
                start_run,
            )
            .add_systems(
                Update,
                update_run_state
                    .run_if(in_state(AppState::Run).or_else(in_state(AppState::Paused))),
            );
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Default)]
pub enum WinState {
    #[default]
    Playing,
    Won,
    Lost,
}

#[derive(Resource, Default)]
pub struct RunState {
    pub run_time: Duration,
    pub win_state: WinState,
    pub live_npcs: u32,
}

fn start_run(mut run_state: ResMut<RunState>) {
    run_state.run_time = Duration::ZERO;
    run_state.win_state = WinState::Playing;
    run_state.live_npcs = 0;
}

fn update_run_state(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut run_state: ResMut<RunState>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    debug_ui: Res<DebugUi>,
) {
    let enter = keyboard.just_pressed(KeyCode::Return) && !debug_ui.has_focus();
    let esc = keyboard.just_pressed(KeyCode::Escape) && !debug_ui.has_focus();
    let playing = run_state.win_state == WinState::Playing;
    if *state.get() == AppState::Paused {
        if playing && enter {
            next_state.set(AppState::Run);
        } else if (!playing && enter) || esc {
            next_state.set(AppState::Menu);
        }
    } else if !playing || esc || enter {
        next_state.set(AppState::Paused);
    } else {
        run_state.run_time += time.delta();
    }
}

fn menu_key_shortcuts(
    keyboard: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    } else if keyboard.just_pressed(KeyCode::Return) {
        next_state.set(AppState::Run);
    }
}

#[derive(Component)]
struct MainMenuUi;

pub const INFINITE_TEMP_COLOR: Color = Color::rgb_linear(
    148. / u8::MAX as f32,
    177. / u8::MAX as f32,
    255. / u8::MAX as f32,
);

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

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
                background_color: NORMAL_BUTTON.into(),
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
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::Run);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut cmd: Commands, q_main_menu: Query<Entity, With<MainMenuUi>>) {
    for entity in &q_main_menu {
        cmd.entity(entity).despawn_recursive();
    }
}
