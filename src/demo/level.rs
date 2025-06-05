//! Spawn the main level.

use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use thiserror::Error;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    demo::player::{PlayerAssets, player},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Reflect)]
pub enum NetworkGraphAssetType {
    Pc(),
    Router(),
    Switch(),
    Server(),
    Firewall(),
    Internet(),
}

impl NetworkGraphAssetType {
    pub fn as_str(&self) -> &str {
        match self {
            NetworkGraphAssetType::Pc() => "pc",
            NetworkGraphAssetType::Router() => "router",
            NetworkGraphAssetType::Switch() => "switch",
            NetworkGraphAssetType::Server() => "server",
            NetworkGraphAssetType::Firewall() => "firewall",
            NetworkGraphAssetType::Internet() => "internet",
        }
    }
    pub fn from_str(s: &str, _params: Vec<String>) -> Result<Self, String> {
        match s {
            "pc" => Ok(NetworkGraphAssetType::Pc()),
            "router" => Ok(NetworkGraphAssetType::Router()),
            "switch" => Ok(NetworkGraphAssetType::Switch()),
            "server" => Ok(NetworkGraphAssetType::Server()),
            "firewall" => Ok(NetworkGraphAssetType::Firewall()),
            "internet" => Ok(NetworkGraphAssetType::Internet()),
            _ => Err("Unknown asset type".to_string()),
        }
    }
}

#[derive(Reflect)]
pub struct NetworkGraphAsset {
    pub asset_type: NetworkGraphAssetType,
    pub name: String,
}

#[derive(Resource, Asset, Reflect, Default)]
#[reflect(Resource)]
pub struct NetworkGraph
{
    pub assets : Vec<NetworkGraphAsset>,
    pub links : Vec<(usize, usize)>, // Links between assets, represented as tuples of indices into the assets property
}

#[derive(Debug, Error)]
pub enum NetworkGraphLoadError {
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Error: Line {0}: {1}")]
    ParseError(i32 /* line number */, String),
    #[error("ObjectParseError: line: {0}, Object {1}: {2}")]
    ObjectParseError(i32 /* line number */, String, String),
    #[error("Invalid directive at line {0}: {1}")]
    InvalidDirective(i32 /* line number */, String),
    #[error("Bad link at line {0}: {1}")]
    BadLinkError(i32 /* line number */, String),
}

impl AssetLoader for NetworkGraph {
    type Asset = NetworkGraph;
    type Settings = ();
    type Error = NetworkGraphLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,

    ) -> Result<Self::Asset, Self::Error>
    {
        let mut string = String::new();
        reader.read_to_string(&mut string).await?;
        let mut graph = NetworkGraph {
            assets: Vec::new(),
            links: Vec::new(),
        };
        let mut line_number = 0;
        for line in string.lines() {
            line_number += 1;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            if trimmed.starts_with("type") {
                // Handle object types
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 2 {
                    return Err(NetworkGraphLoadError::ParseError(
                        line_number,
                        "Invalid type declaration".to_string(),
                    ));
                }
                let object_type = parts[1];
                // Here you can handle the object type, e.g., store it or validate it
                let object_name = parts[2];
                graph.assets.push(NetworkGraphAsset {
                    asset_type: NetworkGraphAssetType::from_str(object_type, parts[3..].iter().map(|s| s.to_string()).collect()).or_else(|err| Err(NetworkGraphLoadError::ObjectParseError(line_number, object_type.to_string(), err)))?,
                    name: object_name.to_string(),
                });
                println!("Found object type: {} with name: {}", object_type, object_name);
            } else if trimmed.starts_with("link") {
                // Handle links
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 3 {
                    return Err(NetworkGraphLoadError::BadLinkError(
                        line_number,
                        "Invalid link declaration".to_string(),
                    ));
                }
                let from = parts[1];
                let to = parts[2];
                let from_index = graph.assets.iter().position(|a| a.name == from)
                    .ok_or_else(|| NetworkGraphLoadError::BadLinkError(line_number, format!("Unknown asset: {}", from)))?;
                let to_index = graph.assets.iter().position(|a| a.name == to)
                    .ok_or_else(|| NetworkGraphLoadError::BadLinkError(line_number, format!("Unknown asset: {}", to)))?;
                graph.links.push((from_index, to_index));
                // Here you can handle the link, e.g., store it in a graph structure
                println!("Found link from {} to {}", from, to);
            } else {
               return Err(NetworkGraphLoadError::InvalidDirective(line_number, trimmed.to_string()))
            }
        }

        Ok(graph)
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
        children![
            player(400.0, &player_assets, &mut texture_atlas_layouts),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            )
        ],
    ));
}

#[cfg(test)]
mod tests {
    use bevy::asset::LoadState;
    use super::*;

    #[test]
    fn test_network_graph_asset_type() {
        let asset_type = NetworkGraphAssetType::from_str("pc", vec![]).unwrap();
        assert_eq!(asset_type.as_str(), "pc");
    }

    #[test]
    fn test_network_graph_asset() {
        let asset = NetworkGraphAsset {
            asset_type: NetworkGraphAssetType::Pc(),
            name: "My PC".to_string(),
        };
        assert_eq!(asset.name, "My PC");
        assert_eq!(asset.asset_type.as_str(), "pc");
    }

    #[test]
    fn test_parsing_network_graph() {
       let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<NetworkGraph>();
        app.init_asset_loader::<NetworkGraph>();
        let handle: Handle<NetworkGraph> = app.world().resource::<AssetServer>().load("levels/test01.txt");
        loop {
            app.update();
            if let LoadState::Loaded = app.world().resource::<AssetServer>().load_state(&handle) {
                break;
            }
            if let LoadState::Failed(err) = app.world().resource::<AssetServer>().load_state(&handle) {
                panic!("Failed to load asset: {:?} - {:?}", handle, err);
            }
        }


        assert!(app.world().resource::<AssetServer>().is_loaded(&handle));
        let graph = app.world().resource::<Assets<NetworkGraph>>().get(&handle).unwrap();

        assert_eq!(graph.assets.len(), 4);
        assert_eq!(graph.assets[0].name, "l01");
        assert_eq!(graph.assets[1].name, "l02");
        assert_eq!(graph.assets[2].name, "l03");
        assert_eq!(graph.assets[3].name, "r01");
        assert_eq!(graph.links.len(), 3);
        assert_eq!(graph.links[0], (0, 3)); // l01 -> r01
        assert_eq!(graph.links[1], (1, 3)); // l02 -> r01
        assert_eq!(graph.links[2], (2, 3)); // l03 -> r01


    }
}
