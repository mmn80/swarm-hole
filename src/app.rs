use std::time::Duration;

use bevy::{app::AppExit, prelude::*};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    Run,
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
            .add_systems(Update, update_run_state.run_if(in_state(AppState::Run)))
            .add_systems(OnExit(AppState::Menu), cleanup_menu)
            .add_systems(OnEnter(AppState::Run), start_run);
    }
}

#[derive(Resource, Default)]
pub struct RunState {
    pub run_time: Duration,
    pub paused: bool,
    pub ended: bool,
    pub won: bool,
    pub live_npcs: u32,
}

pub fn is_running(run_state: Res<RunState>) -> bool {
    !run_state.paused && !run_state.ended
}

fn start_run(mut run_state: ResMut<RunState>) {
    run_state.run_time = Duration::ZERO;
    run_state.ended = false;
    run_state.won = false;
    run_state.live_npcs = 0;
    run_state.paused = false;
}

fn update_run_state(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut run_state: ResMut<RunState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let enter = keyboard.just_released(KeyCode::Return);
    let esc = keyboard.just_released(KeyCode::Escape);
    if run_state.paused {
        if esc {
            run_state.ended = true;
            run_state.won = false;
            run_state.paused = false;
            next_state.set(AppState::Menu);
        } else if enter {
            run_state.paused = false;
        }
    } else if run_state.ended {
        if enter || esc {
            next_state.set(AppState::Menu);
        }
    } else {
        if run_state.live_npcs == 0 && run_state.run_time.as_secs_f32() > 1. {
            run_state.ended = true;
            run_state.won = true;
            run_state.paused = false;
        } else {
            run_state.run_time += time.delta();
            if esc || enter {
                run_state.paused = true;
            }
        }
    }
}

fn menu_key_shortcuts(
    keyboard: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_released(KeyCode::Escape) {
        exit.send(AppExit);
    } else if keyboard.just_released(KeyCode::Return) {
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
                    width: Val::Px(150.),
                    height: Val::Px(65.),
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
                        font_size: 40.0,
                        color: Color::PINK,
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
