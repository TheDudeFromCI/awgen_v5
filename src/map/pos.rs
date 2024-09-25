//! Voxel position types.

use std::fmt;

use bevy::prelude::*;

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
/// Represents an axis-aligned direction in 3D space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceDirection {
    /// The up direction.
    /// This direction points in the positive y-axis direction.
    Up,

    /// The down direction.
    /// This direction points in the negative y-axis direction.
    Down,

    /// The north direction.
    /// This direction points in the negative z-axis direction.
    North,

    /// The south direction.
    /// This direction points in the positive z-axis direction.
    South,

    /// The east direction.
    /// This direction points in the positive x-axis direction.
    East,

    /// The west direction.
    /// This direction points in the negative x-axis direction.
    West,
}

impl FaceDirection {
    /// An array of all six cardinal directions.
    pub const DIRECTIONS: [FaceDirection; 6] = [
        FaceDirection::Up,
        FaceDirection::Down,
        FaceDirection::North,
        FaceDirection::South,
        FaceDirection::East,
        FaceDirection::West,
    ];

    /// Returns the direction that corresponds to the given index.
    ///
    /// This function panics if the given index is not in the range [0, 5].
    #[inline(always)]
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => FaceDirection::Up,
            1 => FaceDirection::Down,
            2 => FaceDirection::North,
            3 => FaceDirection::South,
            4 => FaceDirection::East,
            5 => FaceDirection::West,
            _ => panic!("Invalid direction index: {}", index),
        }
    }

    /// Returns the opposite direction of the given direction.
    #[inline(always)]
    pub fn opposite(self) -> Self {
        match self {
            FaceDirection::Up => FaceDirection::Down,
            FaceDirection::Down => FaceDirection::Up,
            FaceDirection::North => FaceDirection::South,
            FaceDirection::South => FaceDirection::North,
            FaceDirection::East => FaceDirection::West,
            FaceDirection::West => FaceDirection::East,
        }
    }

    /// Returns the index of the direction. This index is used to represent the
    /// direction as an integer in the range [0, 5].
    #[inline(always)]
    pub fn index(self) -> usize {
        match self {
            FaceDirection::Up => 0,
            FaceDirection::Down => 1,
            FaceDirection::North => 2,
            FaceDirection::South => 3,
            FaceDirection::East => 4,
            FaceDirection::West => 5,
        }
    }

    /// Create a quaternion that represents the rotation of an object facing
    /// South, `(-Z)` to this direction.
    ///
    /// When rotating horizontally, the object rotates around the `Y` axis,
    /// preserving the `up` direction. When rotating vertically, the object
    /// rotates around the `X` axis, preserving the `right` direction.
    #[inline(always)]
    pub fn rotation_quat(self) -> Quat {
        let half = std::f32::consts::FRAC_PI_2;
        let full = std::f32::consts::PI;

        /// Quick helper function to create a quaternion from Euler angles.
        #[inline(always)]
        fn euler(x: f32, y: f32, z: f32) -> Quat {
            Quat::from_euler(EulerRot::XYZ, x, y, z)
        }

        match self {
            FaceDirection::Up => euler(-half, 0.0, 0.0),
            FaceDirection::Down => euler(half, 0.0, 0.0),
            FaceDirection::North => euler(0.0, full, 0.0),
            FaceDirection::South => Quat::IDENTITY,
            FaceDirection::East => euler(0.0, half, 0.0),
            FaceDirection::West => euler(0.0, -half, 0.0),
        }
    }
}

impl From<FaceDirection> for IVec3 {
    fn from(value: FaceDirection) -> Self {
        match value {
            FaceDirection::Up => IVec3::new(0, 1, 0),
            FaceDirection::Down => IVec3::new(0, -1, 0),
            FaceDirection::North => IVec3::new(0, 0, -1),
            FaceDirection::South => IVec3::new(0, 0, 1),
            FaceDirection::East => IVec3::new(1, 0, 0),
            FaceDirection::West => IVec3::new(-1, 0, 0),
        }
    }
}

impl From<FaceDirection> for Vec3 {
    fn from(value: FaceDirection) -> Self {
        match value {
            FaceDirection::Up => Vec3::new(0.0, 1.0, 0.0),
            FaceDirection::Down => Vec3::new(0.0, -1.0, 0.0),
            FaceDirection::North => Vec3::new(0.0, 0.0, -1.0),
            FaceDirection::South => Vec3::new(0.0, 0.0, 1.0),
            FaceDirection::East => Vec3::new(1.0, 0.0, 0.0),
            FaceDirection::West => Vec3::new(-1.0, 0.0, 0.0),
        }
    }
}

impl TryFrom<IVec3> for FaceDirection {
    type Error = ();

    fn try_from(value: IVec3) -> Result<Self, Self::Error> {
        match value {
            IVec3 { x: 0, y: 1, z: 0 } => Ok(FaceDirection::Up),
            IVec3 { x: 0, y: -1, z: 0 } => Ok(FaceDirection::Down),
            IVec3 { x: 0, y: 0, z: -1 } => Ok(FaceDirection::North),
            IVec3 { x: 0, y: 0, z: 1 } => Ok(FaceDirection::South),
            IVec3 { x: 1, y: 0, z: 0 } => Ok(FaceDirection::East),
            IVec3 { x: -1, y: 0, z: 0 } => Ok(FaceDirection::West),
            _ => Err(()),
        }
    }
}

impl fmt::Display for FaceDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FaceDirection::Up => write!(f, "Up"),
            FaceDirection::Down => write!(f, "Down"),
            FaceDirection::North => write!(f, "North"),
            FaceDirection::South => write!(f, "South"),
            FaceDirection::East => write!(f, "East"),
            FaceDirection::West => write!(f, "West"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_approx_eq;

    #[test]
    fn rotations() {
        fn test_dir(dir: FaceDirection, forward: Vec3, upward: Vec3) {
            assert_approx_eq!(dir.rotation_quat() * Vec3::Z, forward);
            assert_approx_eq!(dir.rotation_quat() * Vec3::Y, upward);
        }

        test_dir(FaceDirection::Up, Vec3::Y, Vec3::NEG_Z);
        test_dir(FaceDirection::Down, Vec3::NEG_Y, Vec3::Z);

        test_dir(FaceDirection::North, Vec3::NEG_Z, Vec3::Y);
        test_dir(FaceDirection::South, Vec3::Z, Vec3::Y);
        test_dir(FaceDirection::East, Vec3::X, Vec3::Y);
        test_dir(FaceDirection::West, Vec3::NEG_X, Vec3::Y);
    }
}
