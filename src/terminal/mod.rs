mod command;
mod terminal_assets;

use bevy::{
    input::{
        ButtonState,
        keyboard::KeyboardInput,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    picking::hover::HoverMap,
    prelude::*,
    text::LineHeight,
};
use command::Command;
use rand::seq::SliceRandom;
pub use terminal_assets::TerminalAssets;

use crate::{asset_tracking::LoadResource, audio::sound_effect, screens::Screen};

const FONT_SIZE: f32 = 20.0;
const LINE_HEIGHT: f32 = 21.0;
const TERMINAL_CURSOR: &str = "> ";

#[derive(Component)]
struct TerminalContainer;

#[derive(Component, Debug, Default)]
struct TerminalCursor {
    // Holds the current line to eventually be processed
    current_input: String,
    // Cursor location to figure out input/deletion
    cursor_location: usize,
}

#[derive(Component)]
struct TerminalHistory;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
enum TerminalState {
    #[default]
    Ready,
    // Running,
}

/// Helper for creating terminal font
fn terminal_font(terminal_assets: &TerminalAssets) -> TextFont {
    TextFont {
        font: terminal_assets.font.clone(),
        font_size: FONT_SIZE,
        line_height: LineHeight::Px(LINE_HEIGHT),
        ..default()
    }
}

// Helper for creating terminal cursor
fn terminal_cursor(terminal_assets: &TerminalAssets) -> impl Bundle {
    (
        Pickable {
            should_block_lower: false,
            ..default()
        },
        TerminalCursor::default(),
        Text::new(TERMINAL_CURSOR),
        terminal_font(terminal_assets),
    )
}

// Helper for creating terminal history
fn terminal_history(
    input: &str,
    output: &[String],
    terminal_assets: &TerminalAssets,
) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            ..default()
        },
        Pickable {
            should_block_lower: false,
            ..default()
        },
        children![(
            Pickable {
                should_block_lower: false,
                ..default()
            },
            Text::new(format!(
                "{}{}\n{}",
                TERMINAL_CURSOR,
                input,
                output.join("\n")
            )),
            terminal_font(terminal_assets),
        )],
    )
}

// Builds a terminal bundle
pub fn terminal(terminal_assets: &TerminalAssets) -> impl Bundle {
    (
        // Window
        BorderColor(Color::WHITE),
        Node {
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(5.0)),
            box_sizing: BoxSizing::BorderBox,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(5.0)),
            width: Val::Percent(100.0),
            ..default()
        },
        Pickable::IGNORE,
        children![(
            // Container
            Node {
                align_items: AlignItems::Stretch,
                flex_direction: FlexDirection::Column,
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Start,
                overflow: Overflow::scroll_y(),
                width: Val::Percent(100.0),
                ..default()
            },
            TerminalContainer,
            children![
                (
                    Node {
                        align_items: AlignItems::Stretch,
                        flex_direction: FlexDirection::Column,
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    Pickable {
                        should_block_lower: false,
                        ..default()
                    },
                    TerminalHistory,
                ),
                terminal_cursor(terminal_assets)
            ],
        )],
    )
}

/// Handles catching and handling keyboard inputs
/// Mimicking a real terminal as best I can.
fn terminal_input(
    mut commands: Commands,
    mut input_event_reader: EventReader<KeyboardInput>,
    terminal_assets: Res<TerminalAssets>,
    mut terminal_container_query: Query<
        (&mut ComputedNode, &mut ScrollPosition),
        With<TerminalContainer>,
    >,
    mut terminal_cursor_query: Query<&mut TerminalCursor>,
    mut terminal_history_entity_query: Query<Entity, With<TerminalHistory>>,
) {
    let Ok((terminal_container_node, mut terminal_container_scroll)) =
        terminal_container_query.single_mut()
    else {
        return;
    };

    let Ok(mut terminal_cursor) = terminal_cursor_query.single_mut() else {
        return;
    };

    let Ok(terminal_history_entity) = terminal_history_entity_query.single_mut() else {
        return;
    };

    for event in input_event_reader.read() {
        // We only care about button presses right now.
        if event.state == ButtonState::Released {
            continue;
        }

        // Play a sound
        let rng = &mut rand::thread_rng();
        let random_click = terminal_assets.clicks.choose(rng).unwrap().clone();
        commands.spawn(sound_effect(random_click));

        // Execute command
        if event.key_code == KeyCode::Enter {
            // Parse input
            let input_raw = terminal_cursor.current_input.clone();
            let input = terminal_cursor
                .current_input
                .split_whitespace()
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();

            // Build command (or just do a noop if there is no meaningful input)
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

            // Show the input and output as history
            commands
                .entity(terminal_history_entity)
                .with_child(terminal_history(&input_raw, &output, &terminal_assets));

            // Reset cursor to except new input
            terminal_cursor.current_input = String::new();
            terminal_cursor.cursor_location = 0;

            // Scroll to input
            let total_history_newlines = output.len() as f32 + 2.0; // 2 is from input and the spacing between
            let content_height = terminal_container_node.content_size().y;
            let container_height = terminal_container_node.size().y - LINE_HEIGHT;

            if container_height - LINE_HEIGHT < container_height {
                terminal_container_scroll.offset_y =
                    content_height + (LINE_HEIGHT * total_history_newlines)
            }

            continue;
        }

        let cursor_location = terminal_cursor.cursor_location;
        let input_length = terminal_cursor.current_input.len();

        // Backspace (delete character behind)
        if event.key_code == KeyCode::Backspace {
            // Can't delete if we're at the beginning
            if cursor_location == 0 || input_length == 0 {
                continue;
            }

            // If at the end, truncate instead
            if cursor_location == input_length {
                terminal_cursor.current_input.truncate(input_length - 1);
                terminal_cursor.cursor_location -= 1;
                continue;
            }

            // Remove from location
            terminal_cursor.current_input.remove(cursor_location);
            terminal_cursor.cursor_location -= 1;

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

        terminal_cursor
            .current_input
            .insert_str(cursor_location, text.as_str());
        terminal_cursor.cursor_location += 1;
    }
}

/// System for handling scrolling input on the terminal
fn terminal_scrolling(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let dy = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => mouse_wheel_event.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => mouse_wheel_event.y,
        };

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_y -= dy;
                }
            }
        }
    }
}

// Handles displaying text input
fn terminal_text(
    mut terminal_query: Query<(&mut TerminalCursor, &mut Text), Changed<TerminalCursor>>,
) {
    let Ok((terminal, mut text)) = terminal_query.single_mut() else {
        return;
    };
    text.0 = String::new();

    text.0.push_str(TERMINAL_CURSOR);
    text.0.push_str(&terminal.current_input);
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            terminal_input.run_if(in_state(TerminalState::Ready)),
            terminal_scrolling,
            terminal_text,
        )
            .run_if(in_state(Screen::Gameplay)),
    );

    app.init_state::<TerminalState>();

    app.register_type::<TerminalAssets>();
    app.load_resource::<TerminalAssets>();
}
