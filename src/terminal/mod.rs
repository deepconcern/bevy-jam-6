use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
};

use crate::screens::Screen;

const AVAILABLE_COMMANDS: [Command; 2] = [Command::Help, Command::List];
const TERMINAL_CURSOR: &str = "> ";

#[derive(Debug)]
enum Command {
    List,
    Help,
    Invalid,
    Noop,
}

impl Command {
    fn parse(input: &str) -> Command {
        match input.trim() {
            "" => Command::Noop,
            "?" => Command::Help,
            "ls" => Command::List,
            _ => Command::Invalid,
        }
    }

    fn run(&self, args: &[String]) -> Vec<String> {
        let mut output = Vec::new();

        match self {
            Command::Help => {
                if args.is_empty() {
                    output.push("Lol, can't remember your own commands?".to_string());
                    output.push(AVAILABLE_COMMANDS.map(|c| c.to_string()).join(" "));
                } else {
                    output.push(format!(
                        "{}: {}",
                        args[0],
                        match Command::parse(&args[0]) {
                            Command::Help => "Uh... You serious?",
                            Command::List => "List stuff. Like \"virus\" for viruses.",
                            _ => "Man... I don't even know! What nonsense are you asking me?",
                        }
                    ));
                }
            }
            Command::Invalid => output.push(format!(
                "Invalid command, dummy (type ? if you already forgot your own scripts): {}",
                args[0]
            )),
            Command::List => output.push("TODO".to_string()),
            Command::Noop => output.push(String::new()),
        }

        output
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Help => write!(f, "?"),
            Command::List => write!(f, "ls"),
            invalid_command => panic!(
                "Command '{:?}' is not meant to be stringified!",
                invalid_command
            ),
        }
    }
}

#[derive(Component, Debug, Default)]
struct Terminal {
    // Holds the current line to eventually be processed
    current_input: String,
    // Cursor location to figure out input/deletion
    cursor_location: usize,
    // Tuple of historical inputs/outputs
    history: Vec<(String, Vec<String>)>,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum TerminalState {
    #[default]
    Ready,
    // Running,
}

fn setup_terminal(asset_server: Res<AssetServer>, mut commands: Commands) {
    let font_handle = asset_server.load("fonts/VT323-Regular.ttf");

    // Main screen
    commands.spawn((
        BackgroundColor(Color::BLACK),
        Node {
            align_items: AlignItems::Center,
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            ..default()
        },
        StateScoped(Screen::Gameplay),
        children![(
            // Window
            BorderColor(Color::WHITE),
            Node {
                border: UiRect::all(Val::Px(5.0)),
                height: Val::Percent(80.0),
                padding: UiRect::all(Val::Px(5.0)),
                width: Val::Percent(80.0),
                ..default()
            },
            children![(
                // Text
                Terminal::default(),
                Text::new("> █"),
                TextFont::from_font(font_handle),
            )]
        )],
    ));
}

fn terminal_input(
    mut input_event_reader: EventReader<KeyboardInput>,
    mut terminal_query: Query<&mut Terminal>,
) {
    let Ok(mut terminal) = terminal_query.single_mut() else {
        return;
    };

    for event in input_event_reader.read() {
        // We only care about button presses right now.
        if event.state == ButtonState::Released {
            continue;
        }

        // Execute command
        if event.key_code == KeyCode::Enter {
            let input_raw = terminal.current_input.clone();
            let input = terminal
                .current_input
                .split_whitespace()
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();

            let command = if input.is_empty() {
                Command::Noop
            } else {
                Command::parse(&input[0])
            };

            let output = command.run(match command {
                Command::Invalid => &input,
                Command::Noop => &input,
                _ => &input[1..],
            });

            terminal.history.push((input_raw, output));

            terminal.current_input = String::new();
            terminal.cursor_location = 0;

            continue;
        }

        let cursor_location = terminal.cursor_location;
        let input_length = terminal.current_input.len();

        // Backspace (delete character behind)
        if event.key_code == KeyCode::Backspace {
            // Can't delete if we're at the beginning
            if cursor_location == 0 || input_length == 0 {
                continue;
            }

            // If at the end, truncate instead
            if cursor_location == input_length {
                terminal.current_input.truncate(input_length - 1);
                terminal.cursor_location -= 1;
                continue;
            }

            // Remove from location
            terminal.current_input.remove(cursor_location);
            terminal.cursor_location -= 1;

            continue;
        }

        // Del (delete character ahead)
        if event.key_code == KeyCode::Delete {
            // Can't delete if there's nothing to delete
            if input_length == 0 {
                continue;
            }

            // At the end of input, so don't do anything
            if cursor_location == input_length {
                continue;
            }
        }

        // TODO control characters + tab completion

        let Some(text) = &event.text else {
            return;
        };

        terminal
            .current_input
            .insert_str(cursor_location, text.as_str());
        terminal.cursor_location += 1;
    }
}

fn terminal_text(mut terminal_query: Query<(&mut Terminal, &mut Text), Changed<Terminal>>) {
    let Ok((terminal, mut text)) = terminal_query.single_mut() else {
        return;
    };
    text.0 = String::new();

    for (input, output) in &terminal.history {
        // Push input
        text.0.push_str(TERMINAL_CURSOR);
        text.0.push_str(input);
        text.0.push('\n');

        // Push output
        for entry in output {
            text.0.push_str(entry);
            text.0.push('\n');
        }

        // Line separating the next command
        text.0.push('\n');
    }

    text.0.push_str(TERMINAL_CURSOR);
    text.0.push_str(&terminal.current_input);
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), setup_terminal);
    app.add_systems(
        Update,
        (
            terminal_input.run_if(in_state(TerminalState::Ready)),
            terminal_text,
        )
            .run_if(in_state(Screen::Gameplay)),
    );

    app.init_state::<TerminalState>();
}
