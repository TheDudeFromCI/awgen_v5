//! This module implements the [`VoxelChunk`] component and associated logic.

use bevy::prelude::*;
use itertools::Itertools;

use crate::math::{BlockPos, TOTAL_BLOCKS};

/// The data of the blocks within a chunk. This is stored as an enum to allow
/// for data compression when all blocks in the chunk are the same type.
#[derive(Debug, Clone, Component)]
pub enum ChunkData {
    /// The chunk contains only a single block type.
    Single {
        /// The block type in the chunk.
        block: Entity,
    },

    /// The chunk contains multiple block types.
    Multiple {
        /// The blocks in the chunk.
        blocks: Box<[Entity; TOTAL_BLOCKS]>,
    },
}

impl ChunkData {
    /// Creates a new [`ChunkData`] container with all blocks filled with the
    /// given block type.
    pub fn fill(block: Entity) -> Self {
        Self::Single { block }
    }

    /// Replaces the block at the given position within the [`ChunkData`]. This
    /// method does nothing if the block at the given position is already the
    /// same as the given block.
    ///
    /// If the block position is out of the bounds of this chunk, the
    /// coordinates will be wrapped around to the other side of the chunk.
    ///
    /// Returns true if the block was changed, false otherwise.
    pub fn set(&mut self, pos: BlockPos, block: Entity) -> bool {
        if self.get(pos) == block {
            return false;
        }

        match self {
            Self::Single { block: old_block } => {
                let mut blocks = Box::new([*old_block; TOTAL_BLOCKS]);
                blocks[pos.index()] = block;
                *self = Self::Multiple { blocks };
            }
            Self::Multiple { blocks } => {
                blocks[pos.index()] = block;
            }
        }

        true
    }

    /// Returns the block at the given position within the [`ChunkData`].
    pub fn get(&self, pos: BlockPos) -> Entity {
        match self {
            Self::Single { block } => *block,
            Self::Multiple { blocks } => blocks[pos.index()],
        }
    }

    /// Returns the block at the given index within the [`ChunkData`].
    pub fn get_index(&self, index: usize) -> Entity {
        match self {
            Self::Single { block } => *block,
            Self::Multiple { blocks } => blocks[index],
        }
    }

    /// Returns an iterate over all unique blocks in this data container. All
    /// duplicate block entities are removed.
    pub fn iter(&self) -> Box<dyn Iterator<Item = Entity> + '_> {
        match self {
            Self::Single { block } => Box::new(std::iter::once(*block)),
            Self::Multiple { blocks } => Box::new(blocks.iter().sorted().dedup().copied()),
        }
    }

    /// Tries to convert this chunk data into a single block type if all blocks
    /// in the chunk are the same type.
    ///
    /// Returns true if the chunk data was converted, false otherwise. If this
    /// chunk data is already a single block type, this method does nothing and
    /// always returns false.
    pub fn try_convert_to_single(&mut self) -> bool {
        if let Self::Multiple { blocks } = self {
            if blocks.iter().all(|&block| block == blocks[0]) {
                *self = Self::Single { block: blocks[0] };
                return true;
            }
        }

        false
    }
}
