//! A data type for mapping block face occlusion data in the world.

use bevy::prelude::*;
use bitflags::bitflags;

use super::shape::BlockShape;
use crate::map::chunk::ChunkData;
use crate::math::{BlockPos, FaceDirection, TOTAL_BLOCKS};
use crate::utilities::chunk_iter::ChunkIterator;

bitflags! {
    /// Represents the incoming occlusion data for a block face. These flags are
    /// used to determine which faces of a block are occluded by neighboring
    /// blocks.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OccludedBy: u8 {
        /// The upward face is occluded.
        const Up    = 0b00000001;

        /// The downward face is occluded.
        const Down  = 0b00000010;

        /// The northern face is occluded.
        const North = 0b00000100;

        /// The southern face is occluded.
        const South = 0b00001000;

        /// The eastern face is occluded.
        const East  = 0b00010000;

        /// The western face is occluded.
        const West  = 0b00100000;
    }

    /// Represents the outgoing occlusion data for a block. These flags are used
    /// to determine which directions are being occluded by a given block.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Occludes: u8 {
        /// The block occludes the upward direction.
        const Up    = 0b00000001;

        /// The block occludes the downward direction.
        const Down  = 0b00000010;

        /// The block occludes the northern direction.
        const North = 0b00000100;

        /// The block occludes the southern direction.
        const South = 0b00001000;

        /// The block occludes the eastern direction.
        const East  = 0b00010000;

        /// The block occludes the western direction.
        const West  = 0b00100000;
    }
}

impl From<FaceDirection> for OccludedBy {
    fn from(face: FaceDirection) -> Self {
        match face {
            FaceDirection::Up => OccludedBy::Up,
            FaceDirection::Down => OccludedBy::Down,
            FaceDirection::North => OccludedBy::North,
            FaceDirection::South => OccludedBy::South,
            FaceDirection::East => OccludedBy::East,
            FaceDirection::West => OccludedBy::West,
        }
    }
}

impl From<FaceDirection> for Occludes {
    fn from(face: FaceDirection) -> Self {
        match face {
            FaceDirection::Up => Occludes::Up,
            FaceDirection::Down => Occludes::Down,
            FaceDirection::North => Occludes::North,
            FaceDirection::South => Occludes::South,
            FaceDirection::East => Occludes::East,
            FaceDirection::West => Occludes::West,
        }
    }
}

/// A data structure that stores what blocks within a chunk are being occluded
/// by other blocks in the chunk, or by blocks directly outside the chunk
/// bounds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockDataOccludedBy {
    /// The occlusion data for each block in the chunk.
    pub data: Box<[OccludedBy; TOTAL_BLOCKS]>,
}

impl BlockDataOccludedBy {
    /// Creates a new [`BlockDataOccludedBy`] data structure with empty
    /// occlusion data for each block.
    pub fn new() -> Self {
        Self {
            data: Box::new([OccludedBy::empty(); TOTAL_BLOCKS]),
        }
    }

    /// Creates a new [`BlockDataOccludedBy`] data structure from the given
    /// [`BlockData`] containers and [`BlockModel`] query. All blocks outside
    /// the chunk bounds are considered to be empty.
    pub fn from_block_data(blocks: &ChunkData, models: &Query<&BlockShape>) -> Self {
        let occlusion = BlockDataOccludes::from_block_data(blocks, models);
        BlockDataOccludedBy::from_occlusion(&occlusion)
    }

    /// Creates a new [`BlockDataOccludedBy`] data structure from the given
    /// [`BlockDataOccludes`] data structure. All blocks outside the chunk
    /// bounds are considered to be empty.
    pub fn from_occlusion(occlusion: &BlockDataOccludes) -> Self {
        let mut data = BlockDataOccludedBy::new();

        for pos in ChunkIterator::default() {
            let mut occluded_by = OccludedBy::empty();

            if occlusion
                .get(pos.shift(FaceDirection::Up, 1))
                .contains(Occludes::Down)
            {
                occluded_by |= OccludedBy::Up;
            }

            if occlusion
                .get(pos.shift(FaceDirection::Down, 1))
                .contains(Occludes::Up)
            {
                occluded_by |= OccludedBy::Down;
            }

            if occlusion
                .get(pos.shift(FaceDirection::North, 1))
                .contains(Occludes::South)
            {
                occluded_by |= OccludedBy::North;
            }

            if occlusion
                .get(pos.shift(FaceDirection::South, 1))
                .contains(Occludes::North)
            {
                occluded_by |= OccludedBy::South;
            }

            if occlusion
                .get(pos.shift(FaceDirection::East, 1))
                .contains(Occludes::West)
            {
                occluded_by |= OccludedBy::East;
            }

            if occlusion
                .get(pos.shift(FaceDirection::West, 1))
                .contains(Occludes::East)
            {
                occluded_by |= OccludedBy::West;
            }

            data.set(pos, occluded_by);
        }

        data
    }

    /// Gets the occlusion data for the block at the given position. If the
    /// block is outside the chunk bounds, empty occlusion data is returned.
    pub fn get(&self, pos: BlockPos) -> OccludedBy {
        let Some(index) = pos.index_no_wrap() else {
            return OccludedBy::empty();
        };

        self.data[index]
    }

    /// Sets the occlusion data for the block at the given position. If the
    /// block is outside the chunk bounds, this function does nothing.
    pub fn set(&mut self, pos: BlockPos, occludes: OccludedBy) {
        let Some(index) = pos.index_no_wrap() else {
            return;
        };

        self.data[index] = occludes;
    }
}

impl Default for BlockDataOccludedBy {
    fn default() -> Self {
        Self::new()
    }
}

/// This data structure stores what blocks within a chunk are occluding other
/// blocks in the chunk.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockDataOccludes {
    /// The occlusion data for each block in the chunk.
    pub data: Box<[Occludes; TOTAL_BLOCKS]>,
}

impl BlockDataOccludes {
    /// Creates a new [`BlockDataOccludes`] data structure with empty occlusion
    /// data for each block.
    pub fn new() -> Self {
        Self {
            data: Box::new([Occludes::empty(); TOTAL_BLOCKS]),
        }
    }

    /// Creates a new [`BlockDataOccludes`] data structure from the given
    /// [`BlockData`] containers and [`BlockModel`] query.
    pub fn from_block_data(blocks: &ChunkData, models: &Query<&BlockShape>) -> Self {
        let mut data = BlockDataOccludes::new();
        for i in 0 .. TOTAL_BLOCKS {
            let block = blocks.get_index(i);
            data.data[i] = models
                .get(block)
                .map(|model| model.occlusion())
                .unwrap_or(Occludes::empty());
        }

        data
    }

    /// Gets the occlusion data for the block at the given position. If the
    /// block is outside the chunk bounds, empty occlusion data is returned.
    pub fn get(&self, pos: BlockPos) -> Occludes {
        let Some(index) = pos.index_no_wrap() else {
            return Occludes::empty();
        };

        self.data[index]
    }
}

impl Default for BlockDataOccludes {
    fn default() -> Self {
        Self::new()
    }
}
