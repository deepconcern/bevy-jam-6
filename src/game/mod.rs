use bevy::prelude::*;

use crate::{
    screens::Screen,
    terminal::{TerminalAssets, terminal},
};

pub fn spawn_level(mut commands: Commands, terminal_assets: Res<TerminalAssets>) {
    commands.spawn((
        BackgroundColor(Color::BLACK),
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            ..default()
        },
        StateScoped(Screen::Gameplay),
        children![
            Node {
                height: Val::Percent(50.0),
                ..default()
            },
            (
                Node {
                    height: Val::Percent(50.0),
                    ..default()
                },
                children![terminal(&terminal_assets)]
            )
        ],
    ));
}

pub(super) fn plugin(_app: &mut App) {
    // TODO
}
