//! Voxel position types.

use std::fmt;

use bevy::prelude::*;

use super::FaceDirection;

/// The number of bits used to represent a chunk coordinate.
pub const CHUNK_BITS: usize = 4;

/// The size of a chunk in blocks (along one axis).
pub const CHUNK_SIZE: usize = 1 << CHUNK_BITS;

/// The total number of blocks in a chunk (volume).
pub const TOTAL_BLOCKS: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Represents a block position in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    /// The x coordinate of the block.
    pub x: i32,

    /// The y coordinate of the block.
    pub y: i32,

    /// The z coordinate of the block.
    pub z: i32,
}

/// Represents a chunk position in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    /// The x coordinate of the chunk.
    pub x: i32,

    /// The y coordinate of the chunk.
    pub y: i32,

    /// The z coordinate of the chunk.
    pub z: i32,
}

/// A component that represents a position in the world.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Component)]
pub struct Position {
    /// The block position in the world.
    pub block: BlockPos,

    /// The world the position is in.
    pub world: Entity,
}

impl From<BlockPos> for ChunkPos {
    fn from(pos: BlockPos) -> Self {
        Self {
            x: pos.x >> CHUNK_BITS,
            y: pos.y >> CHUNK_BITS,
            z: pos.z >> CHUNK_BITS,
        }
    }
}

impl From<ChunkPos> for BlockPos {
    fn from(pos: ChunkPos) -> Self {
        Self {
            x: pos.x << CHUNK_BITS,
            y: pos.y << CHUNK_BITS,
            z: pos.z << CHUNK_BITS,
        }
    }
}

impl BlockPos {
    /// Creates a new block position.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Returns the index of the block within it's local chunk.
    #[inline(always)]
    pub fn index(self) -> usize {
        let x = self.x as usize & (CHUNK_SIZE - 1);
        let y = self.y as usize & (CHUNK_SIZE - 1);
        let z = self.z as usize & (CHUNK_SIZE - 1);
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }

    /// Returns the index of the block within the chunk. If the block position
    /// is outside of the chunk, this function returns `None`.
    #[inline(always)]
    pub fn index_no_wrap(self) -> Option<usize> {
        if !self.is_in_bounds(ChunkPos::new(0, 0, 0)) {
            return None;
        }

        Some(self.index())
    }

    /// Checks if this block position is within the bounds of the given chunk.
    #[inline(always)]
    pub fn is_in_bounds(self, chunk: ChunkPos) -> bool {
        let min = BlockPos {
            x: chunk.x << CHUNK_BITS,
            y: chunk.y << CHUNK_BITS,
            z: chunk.z << CHUNK_BITS,
        };

        let max = BlockPos {
            x: min.x + CHUNK_SIZE as i32,
            y: min.y + CHUNK_SIZE as i32,
            z: min.z + CHUNK_SIZE as i32,
        };

        self.x >= min.x
            && self.x < max.x
            && self.y >= min.y
            && self.y < max.y
            && self.z >= min.z
            && self.z < max.z
    }

    /// Shift this block position in the given direction by the given number of
    /// units.
    #[inline(always)]
    pub fn shift(self, dir: FaceDirection, amount: u32) -> Self {
        let offset = IVec3::from(dir) * amount as i32;
        BlockPos {
            x: self.x + offset.x,
            y: self.y + offset.y,
            z: self.z + offset.z,
        }
    }

    /// Returns the block position as a `Vec3`.
    #[inline(always)]
    pub fn as_vec3(self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl ChunkPos {
    /// Creates a new chunk position.
    #[inline(always)]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chunk({}, {}, {})", self.x, self.y, self.z)
    }
}
