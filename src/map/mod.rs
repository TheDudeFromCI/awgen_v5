//! This module handles the implementation of the voxel world logic and
//! rendering.

use bevy::prelude::*;
use world::VoxelWorld;

pub mod chunk;
pub mod editor;
pub mod remesh;
pub mod world;

/// The plugin responsible for managing the voxel world.
pub struct VoxelWorldPlugin;
impl Plugin for VoxelWorldPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_resource::<VoxelWorld>()
            .add_plugins((remesh::ChunkRemeshPlugin, editor::MapEditorPlugin));
    }
}

/// This is marker component that indicates that an entity is part of a chunk
/// and is used to test against mouse picking collisions. Any chunks that do
/// not contain this chunk will not be seen by the mouse picker.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct ChunkCollider;
