//! This module handles the implementation for the blocks in the game.

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use uuid::Uuid;

pub mod mesh;
pub mod model;
pub mod occlusion;
pub mod params;
pub mod shape;
pub mod systems;
pub mod tileset;

/// The name of the air block, the default block type for empty space.
pub const AIR_BLOCK_NAME: &str = "Air";

/// The UUID of the air block, the default block type for empty space.
pub const AIR_BLOCK_UUID: Uuid = Uuid::from_u128(0);

/// This plugin adds functionality for working with various block types and
/// their properties.
pub struct BlocksPlugin;
impl Plugin for BlocksPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(
            Update,
            (
                systems::update_rendered_block_model,
                systems::forward_model_changes_to_rendered,
                systems::update_block_model,
                systems::update_custom_block_model_mesh
                    .after_ignore_deferred(systems::update_block_model),
            ),
        )
        .add_systems(Startup, (systems::load_blocks, tileset::load_tilesets));

        embedded_asset!(app_, "prototype.png");
    }
}

/// A component that defines an entity as a block type definition.
///
/// When creating a default block, the UUID is generated randomly.
#[derive(Debug, Component)]
pub struct Block {
    /// The unique identifier for this block type.
    pub uuid: Uuid,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

/// This component can be used to indicate a standalone [`PbrBundle`] entity
/// that reads model data from a block entity.
#[derive(Debug, Component)]
pub struct RenderedBlock {
    /// The block entity to read model data from.
    pub block: Entity,
}
