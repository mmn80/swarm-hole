use bevy::{
    app::AppExit,
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

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
    cmd.spawn(Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Row,
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            Text::new(""),
            Node {
                display: Display::None,
                margin: UiRect::right(Val::Px(20.)),
                ..default()
            },
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Left,
                ..default()
            },
            DebugHelpCommandText,
        ));

        parent.spawn((
            Text::new(""),
            Node {
                display: Display::None,
                ..default()
            },
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Left,
                ..default()
            },
            DebugHelpDescriptionText,
        ));
    });

    cmd.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::FlexEnd,
        justify_content: JustifyContent::Center,
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            Text::new(""),
            Node {
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
            },
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextLayout {
                justify: JustifyText::Center,
                ..default()
            },
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
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    mut debug_ui: ResMut<DebugUi>,
    ev_char: EventReader<KeyboardInput>,
    mut q_text: Query<(&mut Text, &mut Node, &mut Outline), With<DebugUiText>>,
    mut time_since_hidden: Local<Option<f32>>,
    mut cmd: Commands,
) {
    let Ok((mut text, mut node, mut outline)) = q_text.single_mut() else {
        return;
    };

    let mut to_send_cmd = None;
    let mut to_send_param = 0;

    match &debug_ui.state {
        DebugUiCommandParseState::Inactive => {
            if node.display != Display::None {
                if time_since_hidden.is_none() {
                    *time_since_hidden = Some(time.elapsed_secs());
                } else {
                    let time_sec = time_since_hidden.unwrap();
                    if (time.elapsed_secs() - time_sec).abs() > 1. {
                        node.display = Display::None;
                        outline.width = Val::Px(0.);
                        *time_since_hidden = None;
                    }
                }
            }

            if keyboard.just_released(KeyCode::Backquote) {
                debug_ui.state = DebugUiCommandParseState::ReadingCommand("".to_string());
            }
        }

        DebugUiCommandParseState::ReadingCommand(buffer) => {
            node.display = Display::Flex;
            outline.width = Val::Px(2.);
            if keyboard.just_pressed(KeyCode::Escape) {
                debug_ui.state = DebugUiCommandParseState::Inactive;
            } else if !ev_char.is_empty() {
                let new_buffer = append_chars(buffer, ev_char);
                text.0 = format!(":{new_buffer}");

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
                text.0 = format!(":{buffer}");
            }
        }

        DebugUiCommandParseState::ReadingParam(command, key_string, buffer) => {
            if keyboard.just_released(KeyCode::Escape) {
                debug_ui.state = DebugUiCommandParseState::Inactive;
            } else if keyboard.just_released(KeyCode::Enter) {
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
                node.display = Display::Flex;
                text.0 = format!(":{key_string}({new_buffer})");

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
            DebugUiCommand::SpawnRandomNpcs => cmd.queue(crate::npc::SpawnRandomNpcs {
                count: to_send_param as usize,
                distance: 20.,
            }),
            DebugUiCommand::TogglePhysicsDebug => cmd.queue(crate::physics::TogglePhysicsDebug),
            DebugUiCommand::ToggleDebugHelp => cmd.queue(ToggleDebugHelp),
            DebugUiCommand::QuitGame => cmd.queue(QuitGame),
        }
    }
}

fn append_chars(buffer: &String, mut ev_char: EventReader<'_, '_, KeyboardInput>) -> String {
    let mut new_buffer = buffer.clone();
    for ev in ev_char.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Character(char) => {
                for char in char.chars() {
                    if !char.is_control() {
                        new_buffer.push(char);
                    }
                }
            }
            _ => {}
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
                world.query_filtered::<(&mut Text, &mut Node), With<DebugHelpCommandText>>();
            if let Ok((mut text, mut node)) = q_text.single_mut(world) {
                if node.display == Display::None {
                    text.0 = commands;
                    node.display = Display::Flex;
                } else {
                    node.display = Display::None;
                }
            }
        }

        {
            let mut q_text =
                world.query_filtered::<(&mut Text, &mut Node), With<DebugHelpDescriptionText>>();
            if let Ok((mut text, mut node)) = q_text.single_mut(world) {
                if node.display == Display::None {
                    text.0 = help;
                    node.display = Display::Flex;
                } else {
                    node.display = Display::None;
                }
            }
        }
    }
}

pub struct QuitGame;

impl Command for QuitGame {
    fn apply(self, world: &mut World) {
        world.send_event(AppExit::Success);
    }
}
