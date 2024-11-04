//! The [`FaceDirection`] enum represents an axis-aligned direction in 3D space.

use std::fmt;

use bevy::prelude::*;

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

    /// This function attempts to create a `FaceDirection` from a normal vector.
    /// The returned value is based off the closest cardinal direction to the
    /// given normal vector, based on the dot product of the normal vector with
    /// each cardinal direction.
    ///
    /// This function returns `None` if the input normal vector is zero.
    #[inline(always)]
    pub fn from_normal(normal: Vec3) -> Option<Self> {
        let norm = normal.try_normalize()?;

        let north = norm.dot(Vec3::NEG_Z);
        let south = norm.dot(Vec3::Z);
        let up = norm.dot(Vec3::Y);
        let down = norm.dot(Vec3::NEG_Y);
        let east = norm.dot(Vec3::X);
        let west = norm.dot(Vec3::NEG_X);

        let mut best = FaceDirection::Up;
        let mut best_dot = -100.0;

        if north > best_dot {
            best = FaceDirection::North;
            best_dot = north;
        }

        if south > best_dot {
            best = FaceDirection::South;
            best_dot = south;
        }

        if up > best_dot {
            best = FaceDirection::Up;
            best_dot = up;
        }

        if down > best_dot {
            best = FaceDirection::Down;
            best_dot = down;
        }

        if east > best_dot {
            best = FaceDirection::East;
            best_dot = east;
        }

        if west > best_dot {
            best = FaceDirection::West;
        }

        Some(best)
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
