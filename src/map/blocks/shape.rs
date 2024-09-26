use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::occlusion::Occludes;
use crate::math::FaceRotation;
use crate::tileset::TilePos;

/// The shape constructor of a block.
#[derive(Debug, Default, Clone, Component, Serialize, Deserialize)]
pub enum BlockShape {
    /// No model.
    #[default]
    None,

    /// A standard cubic block.
    Cube {
        /// The tileset of the block.
        tileset: String,

        /// The texture properties of the top face of the block.
        top: BlockFace,

        /// The texture properties of the bottom face of the block.
        bottom: BlockFace,

        /// The texture properties of the north face of the block.
        north: BlockFace,

        /// The texture properties of the south face of the block.
        south: BlockFace,

        /// The texture properties of the east face of the block.
        east: BlockFace,

        /// The texture properties of the west face of the block.
        west: BlockFace,
    },

    /// A block with a custom shape.
    Custom {
        /// The asset path of the block model.
        asset: String,
    },
}

impl BlockShape {
    /// Gets what surrounding blocks are occluded by this block. Note that this
    /// method does not check tileset transparency and assumes that the block
    /// model is always is opaque. A tileset that contains transparent textures
    /// should always be considered as never occluding.
    ///
    /// This method also assumes that all custom models as fully transparent.
    #[inline(always)]
    pub fn occlusion(&self) -> Occludes {
        match self {
            BlockShape::None => Occludes::empty(),
            BlockShape::Cube { .. } => Occludes::all(),
            BlockShape::Custom { .. } => Occludes::empty(),
        }
    }
}

/// The texture properties of a face of a block.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BlockFace {
    /// The tile position of the block face within the tileset.
    pub tile: TilePos,

    /// The rotation of the texture.
    pub rotation: FaceRotation,

    /// Whether the texture is mirrored along the x-axis. (Before rotation)
    pub mirror_x: bool,

    /// Whether the texture is mirrored along the y-axis. (Before rotation)
    pub mirror_y: bool,
}
