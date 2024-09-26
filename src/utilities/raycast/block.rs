//! Implements the voxel raycast system parameter.

use bevy::ecs::system::SystemParam;
use bevy::math::bounding::RayCast3d;
use bevy::prelude::*;

use super::VoxelIterator;
use crate::map::blocks::model::BlockModel;
use crate::map::chunk::ChunkData;
use crate::map::world::VoxelWorld;
use crate::math::{BlockPos, ChunkPos, FaceDirection};

/// A system parameter that provides the ability to perform a raycast on a voxel
/// grid.
#[derive(SystemParam)]
pub struct VoxelRaycast<'w, 's> {
    /// The query for voxel world entities.
    worlds: Query<'w, 's, &'static VoxelWorld>,

    /// The query for chunk data components.
    chunks: Query<'w, 's, &'static ChunkData>,

    /// The query for block model components.
    blocks: Query<'w, 's, &'static BlockModel>,
}

impl<'w, 's> VoxelRaycast<'w, 's> {
    /// Casts a ray into the voxel world and returns the first block that was
    /// hit, or `None` if no block was hit.
    pub fn raycast(&self, world_id: Entity, raycast: RayCast3d) -> Option<VoxelRaycastHit> {
        let Ok(world) = self.worlds.get(world_id) else {
            return None;
        };

        let mut chunk_pos: Option<ChunkPos> = None;
        let mut chunk_buf = None;

        for (block_pos, face) in VoxelIterator::new(raycast.origin, raycast.direction)
            .with_max_distance(raycast.max)
            .skip(1)
        {
            if Some(block_pos.into()) != chunk_pos {
                chunk_pos = Some(block_pos.into());

                let Some(chunk_id) = world.get_chunk(block_pos.into()) else {
                    chunk_buf = None;
                    continue;
                };

                chunk_buf = self.chunks.get(chunk_id).ok();
            }

            let Some(chunk) = &chunk_buf else {
                continue;
            };

            let block_id = chunk.get(block_pos);
            let Ok(model) = self.blocks.get(block_id) else {
                continue;
            };

            let Some(mut bounds) = model.get_bounds() else {
                continue;
            };

            bounds.min += block_pos.as_vec3a();
            bounds.max += block_pos.as_vec3a();

            let Some(block_dist) = raycast.aabb_intersection_at(&bounds) else {
                continue;
            };

            let hit_pos = (raycast.origin + raycast.direction * block_dist).into();
            let face = face.unwrap();

            return Some(VoxelRaycastHit {
                world: world_id,
                block: block_pos,
                face,
                distance: block_dist,
                hit_pos,
            });
        }

        None
    }
}

/// Represents the result of a voxel raycast.
#[derive(Debug, Clone)]
pub struct VoxelRaycastHit {
    /// The entity of the world that was targeted.
    pub world: Entity,

    /// The position of the block that was hit.
    pub block: BlockPos,

    /// The face of the block that was hit.
    pub face: FaceDirection,

    /// The distance from the ray origin to the hit point.
    pub distance: f32,

    /// The hit position in world space.
    pub hit_pos: Vec3,
}
