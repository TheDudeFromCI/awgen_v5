//! This module implements the handling for construction of block models.

use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;

use super::mesh::BlockMesh;

/// The model definition of a block, as defined by the block's mesh and
/// material.
#[derive(Debug, Default, Clone, Component)]
pub enum BlockModel {
    /// The block has no model.
    #[default]
    None,

    /// The block has a primitive shape and can be used in the construction of
    /// static chunk meshes.
    Primitive {
        /// The material of the block.
        material: Handle<StandardMaterial>,

        /// The mesh of the block.
        mesh: Box<BlockMesh>,

        /// The bounds of the block mesh. Used for raycasting.
        bounds: Aabb3d,
    },

    /// The block has a custom model and is added as a child of a chunk entity.
    Custom {
        /// The material of the block.
        material: Handle<StandardMaterial>,

        /// The mesh of the block.
        mesh: Handle<Mesh>,

        /// The bounds of the mesh. Used for raycasting.
        bounds: Aabb3d,
    },
}

impl BlockModel {
    /// Gets the bounds of the block model. Returns `None` if the block has no
    /// model.
    pub fn get_bounds(&self) -> Option<Aabb3d> {
        match self {
            BlockModel::None => None,
            BlockModel::Primitive { bounds, .. } => Some(*bounds),
            BlockModel::Custom { bounds, .. } => Some(*bounds),
        }
    }
}
