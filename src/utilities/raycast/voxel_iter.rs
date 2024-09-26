//! This module contains a voxel iterator implementation, that allows iterating
//! over a raycast on a voxel grid.

use bevy::math::{Dir3, IVec3, Vec3};

use crate::math::{BlockPos, FaceDirection};

/// An iterator that iterates over the voxels intersected by a ray. The iterator
/// returns the block position and the face of each voxel that was intersected.
/// The face is `None` for the first voxel.
#[derive(Debug, Clone)]
pub struct VoxelIterator {
    /// The direction of the ray.
    dir: Dir3,

    /// The current block position.
    block: BlockPos,

    /// The step direction.
    step: IVec3,

    /// The maximum t values for each axis.
    t_max: Vec3,

    /// The t delta values for each axis.
    t_delta: Vec3,

    /// The minimum block position.
    min_block: BlockPos,

    /// The maximum block position.
    max_block: BlockPos,

    /// The face direction of the current voxel intersection.
    face: Option<FaceDirection>,

    /// The maximum distance of the ray.
    max_distance: f32,
}

impl VoxelIterator {
    /// Creates a new voxel iterator with the given starting point and
    /// direction.
    pub fn new(point: impl Into<Vec3>, dir: impl Into<Dir3>) -> Self {
        let point = point.into();
        let dir = dir.into();

        let step = dir.signum().as_ivec3();
        Self {
            dir,
            block: BlockPos::from_vec3(point),
            step,
            t_max: vec_intbound(point, *dir),
            t_delta: step.as_vec3() / *dir,
            min_block: BlockPos::new(i32::MIN, i32::MIN, i32::MIN),
            max_block: BlockPos::new(i32::MAX, i32::MAX, i32::MAX),
            face: None,
            max_distance: f32::MAX,
        }
    }

    /// Updates the region of the iterator. This does not reset the iterator.
    /// Returns self for chaining.
    pub fn within_region(mut self, min: BlockPos, max: BlockPos) -> Self {
        self.min_block = min;
        self.max_block = max;
        self
    }

    /// Updates the maximum distance of the ray. This does not reset the
    /// iterator. Returns self for chaining.
    pub fn with_max_distance(mut self, max_distance: f32) -> Self {
        let x = self.dir.x * self.dir.x + self.dir.y * self.dir.y + self.dir.z * self.dir.z;
        self.max_distance = max_distance / x.sqrt();
        self
    }
}

impl Iterator for VoxelIterator {
    type Item = (BlockPos, Option<FaceDirection>);

    #[allow(clippy::collapsible_else_if)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.max_distance < 0.0 {
            return None;
        }

        if !self.block.is_in_bounds(self.min_block, self.max_block) {
            return None;
        }

        let current_block = self.block;
        let current_face = self.face;

        if self.t_max.x < self.t_max.y {
            if self.t_max.x < self.t_max.z {
                if self.t_max.x > self.max_distance {
                    self.max_distance = -1.0;
                }
                self.block.x += self.step.x;
                self.t_max.x += self.t_delta.x;
                self.face = Some(IVec3::new(-self.step.x, 0, 0).try_into().unwrap());
            } else {
                if self.t_max.z > self.max_distance {
                    self.max_distance = -1.0;
                }
                self.block.z += self.step.z;
                self.t_max.z += self.t_delta.z;
                self.face = Some(IVec3::new(0, 0, -self.step.z).try_into().unwrap());
            }
        } else {
            if self.t_max.y < self.t_max.z {
                if self.t_max.y > self.max_distance {
                    self.max_distance = -1.0;
                }
                self.block.y += self.step.y;
                self.t_max.y += self.t_delta.y;
                self.face = Some(IVec3::new(0, -self.step.y, 0).try_into().unwrap());
            } else {
                if self.t_max.z > self.max_distance {
                    self.max_distance = -1.0;
                }
                self.block.z += self.step.z;
                self.t_max.z += self.t_delta.z;
                self.face = Some(IVec3::new(0, 0, -self.step.z).try_into().unwrap());
            }
        }

        Some((current_block, current_face))
    }
}

/// Finds the smallest positive t such that s + t * ds is an integer.
fn intbound(s: f32, ds: f32) -> f32 {
    if ds < 0.0 {
        intbound(-s, -ds)
    } else {
        let g = (s % 1.0 + 1.0) % 1.0;
        (1.0 - g) / ds
    }
}

/// Calculates the integer bounds for each element of a vector.
fn vec_intbound(s: Vec3, ds: Vec3) -> Vec3 {
    Vec3::new(
        intbound(s.x, ds.x),
        intbound(s.y, ds.y),
        intbound(s.z, ds.z),
    )
}
