use bevy::prelude::*;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugUi>()
            .add_event::<DebugUiEvent>()
            .add_systems(Startup, setup_debug_ui)
            .add_systems(Update, (listen_debug_commands, update_debug_ui));
    }
}

#[derive(Resource)]
pub struct DebugUi {
    commands: Vec<DebugUiCommandConfig>,
    state: DebugUiCommandParseState,
}

impl Default for DebugUi {
    fn default() -> Self {
        Self {
            commands: vec![DebugUiCommandConfig::new_simple(
                DebugUiCommand::TogglePhysicsDebug,
                "p",
            )],
            state: DebugUiCommandParseState::Inactive,
        }
    }
}

impl DebugUi {
    pub fn register_command(&mut self, command: DebugUiCommandConfig) {
        self.commands.push(command);
    }

    pub fn has_focus(&self) -> bool {
        self.state != DebugUiCommandParseState::Inactive
    }
}

#[derive(PartialEq, Eq)]
enum DebugUiCommandParseState {
    Inactive,
    ReadingCommand(String),
    ReadingParam(DebugUiCommand, String),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DebugUiCommand {
    TogglePhysicsDebug,
}

pub struct DebugUiCommandConfig {
    pub command: DebugUiCommand,
    pub key_string: String,
    pub has_param: bool,
}

impl DebugUiCommandConfig {
    pub fn new(command: DebugUiCommand, key_string: &str, has_param: bool) -> Self {
        Self {
            command,
            key_string: key_string.to_string(),
            has_param,
        }
    }

    pub fn new_simple(command: DebugUiCommand, key_string: &str) -> Self {
        Self {
            command,
            key_string: key_string.to_string(),
            has_param: false,
        }
    }
}

#[derive(Event)]
pub struct DebugUiEvent {
    pub command: DebugUiCommand,
    pub param: i32,
}

fn listen_debug_commands(
    keyboard: Res<Input<KeyCode>>,
    mut debug_ui: ResMut<DebugUi>,
    mut ev_char: EventReader<ReceivedCharacter>,
    mut ev_debug_ui: EventWriter<DebugUiEvent>,
) {
    match &debug_ui.state {
        DebugUiCommandParseState::Inactive => {
            if keyboard.just_released(KeyCode::Grave) {
                debug_ui.state = DebugUiCommandParseState::ReadingCommand("".to_string());
            }
        }
        DebugUiCommandParseState::ReadingCommand(buffer) => {
            if keyboard.just_pressed(KeyCode::Escape) {
                debug_ui.state = DebugUiCommandParseState::Inactive;
            } else if !ev_char.is_empty() {
                let mut new_buffer = buffer.clone();
                for ev in ev_char.read() {
                    if !ev.char.is_control() {
                        new_buffer.push(ev.char);
                    }
                }
                if let Some((command, has_param)) = debug_ui
                    .commands
                    .iter()
                    .find(|cmd| cmd.key_string == new_buffer)
                    .map(|cmd| (cmd.command, cmd.has_param))
                {
                    if has_param {
                        debug_ui.state =
                            DebugUiCommandParseState::ReadingParam(command, "".to_string());
                    } else {
                        ev_debug_ui.send(DebugUiEvent { command, param: 0 });
                        debug_ui.state = DebugUiCommandParseState::Inactive;
                    }
                } else {
                    debug_ui.state = DebugUiCommandParseState::ReadingCommand(new_buffer);
                }
            }
        }
        DebugUiCommandParseState::ReadingParam(command, buffer) => {
            if keyboard.just_pressed(KeyCode::Escape) {
                debug_ui.state = DebugUiCommandParseState::Inactive;
            } else if keyboard.just_pressed(KeyCode::Return) {
                if let Ok(param) = buffer.parse::<i32>() {
                    ev_debug_ui.send(DebugUiEvent {
                        command: *command,
                        param,
                    });
                    debug_ui.state = DebugUiCommandParseState::Inactive;
                } else {
                    error!("Parsing string '{buffer}' as number failed.");
                    debug_ui.state = DebugUiCommandParseState::Inactive;
                }
            } else if !ev_char.is_empty() {
                let mut new_buffer = buffer.clone();
                for ev in ev_char.read() {
                    if !ev.char.is_control() {
                        new_buffer.push(ev.char);
                    }
                }
                DebugUiCommandParseState::ReadingParam(*command, new_buffer);
            }
        }
    }
}

#[derive(Component)]
struct DebugUiText;

fn setup_debug_ui(mut cmd: Commands) {
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
                width: Val::Px(2.),
                offset: Val::Px(6.),
                color: Color::WHITE,
            },
            DebugUiText,
        ));
    });
}

fn update_debug_ui(
    debug_ui: Res<DebugUi>,
    mut query: Query<(&mut Text, &mut Style), With<DebugUiText>>,
) {
    for (mut text, mut style) in &mut query {
        match &debug_ui.state {
            DebugUiCommandParseState::Inactive => {
                style.display = Display::None;
            }
            DebugUiCommandParseState::ReadingCommand(buffer) => {
                style.display = Display::Flex;
                text.sections[0].value = format!("{buffer}");
            }
            DebugUiCommandParseState::ReadingParam(_command, buffer) => {
                style.display = Display::Flex;
                text.sections[0].value = format!("{buffer}");
            }
        }
    }
}
