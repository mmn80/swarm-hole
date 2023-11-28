use bevy::{app::AppExit, ecs::system::Command, prelude::*};

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugUi>()
            .add_systems(Startup, setup_debug_ui)
            .add_systems(Update, process_debug_commands);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DebugUiCommand {
    SpawnRandomNpcs,
    TogglePhysicsDebug,
    ToggleDebugHelp,
    QuitGame,
}

#[derive(Resource)]
pub struct DebugUi {
    commands: Vec<DebugUiCommandConfig>,
    state: DebugUiCommandParseState,
}

impl Default for DebugUi {
    fn default() -> Self {
        Self {
            commands: vec![
                DebugUiCommandConfig::new_param(
                    DebugUiCommand::SpawnRandomNpcs,
                    "n",
                    "spawn a given number of NPCs",
                ),
                DebugUiCommandConfig::new(
                    DebugUiCommand::TogglePhysicsDebug,
                    "p",
                    "toggle physics debug draw",
                ),
                DebugUiCommandConfig::new(
                    DebugUiCommand::ToggleDebugHelp,
                    "h",
                    "toggle showing this help",
                ),
                DebugUiCommandConfig::new(DebugUiCommand::QuitGame, "q", "quit game"),
            ],
            state: DebugUiCommandParseState::Inactive,
        }
    }
}

impl DebugUi {
    pub fn has_focus(&self) -> bool {
        self.state != DebugUiCommandParseState::Inactive
    }
}

#[derive(PartialEq, Eq)]
enum DebugUiCommandParseState {
    Inactive,
    ReadingCommand(String),
    ReadingParam(DebugUiCommand, String, String),
}

pub struct DebugUiCommandConfig {
    pub command: DebugUiCommand,
    pub key_string: String,
    pub has_param: bool,
    pub help: String,
}

impl DebugUiCommandConfig {
    pub fn new_param(command: DebugUiCommand, key_string: &str, help: &str) -> Self {
        Self {
            command,
            key_string: key_string.to_string(),
            has_param: true,
            help: help.to_string(),
        }
    }

    pub fn new(command: DebugUiCommand, key_string: &str, help: &str) -> Self {
        Self {
            command,
            key_string: key_string.to_string(),
            has_param: false,
            help: help.to_string(),
        }
    }
}

#[derive(Component)]
struct DebugUiText;

#[derive(Component)]
struct DebugHelpCommandText;

#[derive(Component)]
struct DebugHelpDescriptionText;

fn setup_debug_ui(mut cmd: Commands) {
    cmd.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 20.0,
                    ..default()
                },
            )
            .with_text_alignment(TextAlignment::Left)
            .with_style(Style {
                display: Display::None,
                margin: UiRect::right(Val::Px(20.)),
                ..default()
            }),
            DebugHelpCommandText,
        ));

        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 20.0,
                    ..default()
                },
            )
            .with_text_alignment(TextAlignment::Left)
            .with_style(Style {
                display: Display::None,
                ..default()
            }),
            DebugHelpDescriptionText,
        ));
    });

    cmd.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Auto,
                    bottom: Val::Percent(2.),
                },
                ..default()
            }),
            Outline {
                width: Val::Px(0.),
                offset: Val::Px(6.),
                color: Color::WHITE,
            },
            DebugUiText,
        ));
    });
}

fn process_debug_commands(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time<Real>>,
    mut debug_ui: ResMut<DebugUi>,
    ev_char: EventReader<ReceivedCharacter>,
    mut q_text: Query<(&mut Text, &mut Style, &mut Outline), With<DebugUiText>>,
    mut time_since_hidden: Local<Option<f32>>,
    mut cmd: Commands,
) {
    let Ok((mut text, mut style, mut outline)) = q_text.get_single_mut() else {
        return;
    };

    let mut to_send_cmd = None;
    let mut to_send_param = 0;

    match &debug_ui.state {
        DebugUiCommandParseState::Inactive => {
            if style.display != Display::None {
                if time_since_hidden.is_none() {
                    *time_since_hidden = Some(time.elapsed_seconds());
                } else {
                    let time_sec = time_since_hidden.unwrap();
                    if (time.elapsed_seconds() - time_sec).abs() > 1. {
                        style.display = Display::None;
                        outline.width = Val::Px(0.);
                        *time_since_hidden = None;
                    }
                }
            }

            if keyboard.just_released(KeyCode::Grave) {
                debug_ui.state = DebugUiCommandParseState::ReadingCommand("".to_string());
            }
        }

        DebugUiCommandParseState::ReadingCommand(buffer) => {
            style.display = Display::Flex;
            outline.width = Val::Px(2.);
            if keyboard.just_pressed(KeyCode::Escape) {
                debug_ui.state = DebugUiCommandParseState::Inactive;
            } else if !ev_char.is_empty() {
                let new_buffer = append_chars(buffer, ev_char);
                text.sections[0].value = format!(":{new_buffer}");

                if let Some((command, has_param)) = debug_ui
                    .commands
                    .iter()
                    .find(|cmd| cmd.key_string == new_buffer)
                    .map(|cmd| (cmd.command, cmd.has_param))
                {
                    if has_param {
                        debug_ui.state = DebugUiCommandParseState::ReadingParam(
                            command,
                            new_buffer,
                            "".to_string(),
                        );
                    } else {
                        to_send_cmd = Some(command);
                        to_send_param = 0;
                        debug_ui.state = DebugUiCommandParseState::Inactive;
                    }
                } else {
                    debug_ui.state = DebugUiCommandParseState::ReadingCommand(new_buffer);
                }
            } else {
                text.sections[0].value = format!(":{buffer}");
            }
        }

        DebugUiCommandParseState::ReadingParam(command, key_string, buffer) => {
            if keyboard.just_released(KeyCode::Escape) {
                debug_ui.state = DebugUiCommandParseState::Inactive;
            } else if keyboard.just_released(KeyCode::Return) {
                if let Ok(param) = buffer.parse::<i32>() {
                    to_send_cmd = Some(*command);
                    to_send_param = param;
                    debug_ui.state = DebugUiCommandParseState::Inactive;
                } else {
                    error!("Parsing string '{buffer}' as number failed.");
                    debug_ui.state = DebugUiCommandParseState::Inactive;
                }
            } else if !ev_char.is_empty() {
                let new_buffer = append_chars(buffer, ev_char);
                style.display = Display::Flex;
                text.sections[0].value = format!(":{key_string}({new_buffer})");

                debug_ui.state = DebugUiCommandParseState::ReadingParam(
                    *command,
                    key_string.clone(),
                    new_buffer,
                );
            }
        }
    }

    if let Some(command) = to_send_cmd {
        match command {
            DebugUiCommand::SpawnRandomNpcs => cmd.add(crate::npc::SpawnRandomNpcs {
                count: to_send_param as usize,
                distance: 20.,
            }),
            DebugUiCommand::TogglePhysicsDebug => cmd.add(crate::physics::TogglePhysicsDebug),
            DebugUiCommand::ToggleDebugHelp => cmd.add(ToggleDebugHelp),
            DebugUiCommand::QuitGame => cmd.add(QuitGame),
        }
    }
}

fn append_chars(buffer: &String, mut ev_char: EventReader<'_, '_, ReceivedCharacter>) -> String {
    let mut new_buffer = buffer.clone();
    for ev in ev_char.read() {
        if !ev.char.is_control() {
            new_buffer.push(ev.char);
        }
    }
    new_buffer
}

pub struct ToggleDebugHelp;

impl Command for ToggleDebugHelp {
    fn apply(self, world: &mut World) {
        let (commands, help) = {
            let debug_ui = world.resource::<DebugUi>();
            let commands = debug_ui
                .commands
                .iter()
                .map(|cmd| {
                    format!(
                        "{}{}\n",
                        cmd.key_string,
                        if cmd.has_param { "(n)" } else { "" },
                    )
                })
                .collect::<Vec<_>>()
                .concat();
            let help = debug_ui
                .commands
                .iter()
                .map(|cmd| format!("- {}\n", cmd.help))
                .collect::<Vec<_>>()
                .concat();
            (commands, help)
        };

        {
            let mut q_text =
                world.query_filtered::<(&mut Text, &mut Style), With<DebugHelpCommandText>>();
            if let Ok((mut text, mut style)) = q_text.get_single_mut(world) {
                if style.display == Display::None {
                    text.sections[0].value = commands;
                    style.display = Display::Flex;
                } else {
                    style.display = Display::None;
                }
            }
        }

        {
            let mut q_text =
                world.query_filtered::<(&mut Text, &mut Style), With<DebugHelpDescriptionText>>();
            if let Ok((mut text, mut style)) = q_text.get_single_mut(world) {
                if style.display == Display::None {
                    text.sections[0].value = help;
                    style.display = Display::Flex;
                } else {
                    style.display = Display::None;
                }
            }
        }
    }
}

pub struct QuitGame;

impl Command for QuitGame {
    fn apply(self, world: &mut World) {
        world.send_event(AppExit);
    }
}
