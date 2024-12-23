//! This module implements tileset loading and management.

use bevy::prelude::*;
use bevy::render::texture::{ImageLoaderSettings, ImageSampler};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The total number of tiles in a tileset across one axis.
///
/// The entire tileset is a square image with `TILESET_LENGTH * TILESET_LENGTH`
/// tiles in a 2D grid.
pub const TILESET_LENGTH: usize = 16;

/// The name of the prototype tileset, the default tileset used for testing.
pub const PROTOTYPE_TILESET_NAME: &str = "Prototype";

/// The UUID of the prototype tileset, the default tileset used for testing.
pub const PROTOTYPE_TILESET_UUID: Uuid = Uuid::from_u128(0);

/// The asset path to the prototype tileset image.
pub const PROTOTYPE_TILESET_PATH: &str = "embedded://awgen/blocks/prototype.png";

/// A marker component that defines an entity as a tileset definition.
///
/// When creating a default tileset, the UUID is generated randomly.
#[derive(Debug, Component)]
pub struct Tileset {
    /// The unique identifier for this tileset.
    pub uuid: Uuid,
}

impl Default for Tileset {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

/// A bundle that defines the components of a tileset.
#[derive(Debug, Default, Bundle)]
pub struct TilesetBundle {
    /// A marker component that defines an entity as a tileset definition.
    pub tileset: Tileset,

    /// The name of the tileset.
    pub name: Name,

    /// The tileset image handle.
    ///
    /// Ideally, this should be a weak handle to prevent the image from
    /// remaining in RAM after it has been loaded.
    pub image: Handle<Image>,

    /// The material used to render the tileset.
    pub material: Handle<StandardMaterial>,
}

/// A position of a tile in a tileset.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TilePos {
    /// The x-coordinate of the tile.
    x: u8,

    /// The y-coordinate of the tile.
    y: u8,
}

impl TilePos {
    /// Creates a new tile position with the given coordinates.
    ///
    /// This function panics if the given coordinates are out of bounds. Values
    /// *must* be less than `TILESET_LENGTH`.
    pub fn new(x: u8, y: u8) -> Self {
        if x >= TILESET_LENGTH as u8 || y >= TILESET_LENGTH as u8 {
            panic!(
                "Tile TILESET_LENGTH ({}, {}) is out of bounds for a tile set with {} tiles",
                x, y, TILESET_LENGTH
            );
        }

        Self { x, y }
    }

    /// Transforms a UV coordinate in the range of [0, 1] to the UV coordinate
    /// of the tile in the texture atlas.
    pub fn transform_uv(self, uv: Vec2) -> Vec2 {
        let size = 1.0 / TILESET_LENGTH as f32;
        Vec2::new(
            uv.x * size + self.x as f32 * size,
            uv.y * size + self.y as f32 * size,
        )
    }
}

/// This system is called on startup to load all tilesets into the world.
pub fn load_tilesets(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    // prototype tileset
    let tileset_image = asset_server.load_with_settings(
        PROTOTYPE_TILESET_PATH,
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    commands.spawn(TilesetBundle {
        tileset: Tileset {
            uuid: PROTOTYPE_TILESET_UUID,
        },
        name: Name::new(PROTOTYPE_TILESET_NAME),
        image: tileset_image.clone(),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(tileset_image),
            perceptual_roughness: 1.0,
            ..default()
        }),
    });

    load_tileset(&asset_server, &mut materials, &mut commands, "overworld");
}

/// Loads the tileset with the given name.
fn load_tileset(
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands,
    name: &str,
) {
    let tileset_image = asset_server.load_with_settings(
        format!("project://tilesets/{name}.png"),
        |settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        },
    );
    commands.spawn(TilesetBundle {
        name: Name::new(name.to_string()),
        image: tileset_image.clone(),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(tileset_image),
            perceptual_roughness: 1.0,
            ..default()
        }),
        ..default()
    });
}

/// A struct that represents a tileset definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetDefinition {
    /// The UUID of the tileset.
    pub uuid: Uuid,

    /// The name of the tileset.
    pub name: String,
}
