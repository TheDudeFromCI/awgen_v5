//! This module handles the implementation for the blocks in the game.

use bevy::prelude::*;

pub mod mesh;
pub mod model;
pub mod occlusion;
pub mod shape;
mod systems;

pub struct BlocksPlugin;
impl Plugin for BlocksPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(
            Update,
            (
                systems::update_rendered_block_model,
                systems::forward_model_changes_to_rendered,
                systems::update_block_model,
            ),
        );
    }
}

/// A marker component that defines an entity as a block type definition.
#[derive(Debug, Default, Component)]
pub struct Block;

/// This component can be used to indicate a standalone [`PbrBundle`] entity
/// that reads model data from a block entity.
#[derive(Debug, Component)]
pub struct RenderedBlock {
    /// The block entity to read model data from.
    pub block: Entity,
}
