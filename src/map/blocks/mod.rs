//! This module handles the implementation for the blocks in the game.

use bevy::prelude::*;

pub mod model;
pub mod occlusion;

/// A marker component that defines an entity as a block type definition.
#[derive(Debug, Default, Component)]
pub struct Block;

/// A marker component that defines an entity as a tileset definition.
#[derive(Debug, Default, Component)]
pub struct Tileset;
