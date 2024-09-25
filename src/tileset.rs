//! This module implements tileset loading and management.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// The total number of tiles in a tileset across one axis.
///
/// The entire tileset is a square image with `TILESET_LENGTH * TILESET_LENGTH`
/// tiles in a 2D grid.
pub const TILESET_LENGTH: usize = 16;

/// A marker component that defines an entity as a tileset definition.
#[derive(Debug, Default, Component)]
pub struct Tileset;

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

    /// Gets the x-coordinate of the tile in the atlas. This value is in the
    /// range of [0, `TILE_COUNT`).
    pub fn x(&self) -> u8 {
        self.x
    }

    /// Gets the y-coordinate of the tile in the atlas. This value is in the
    /// range of [0, `TILE_COUNT`).
    pub fn y(&self) -> u8 {
        self.y
    }

    /// Returns the index of the tile in the texture atlas.
    pub fn index(self) -> usize {
        self.y as usize * TILESET_LENGTH + self.x as usize
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
