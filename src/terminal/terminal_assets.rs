use bevy::prelude::*;

#[derive(Asset, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct TerminalAssets {
    #[dependency]
    pub clicks: Vec<Handle<AudioSource>>,
    #[dependency]
    pub font: Handle<Font>,
}

impl FromWorld for TerminalAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            clicks: vec![
                assets.load("audio/sound_effects/keypress-001.wav"),
                assets.load("audio/sound_effects/keypress-002.wav"),
                assets.load("audio/sound_effects/keypress-003.wav"),
            ],
            font: assets.load("fonts/VT323-Regular.ttf"),
        }
    }
}
