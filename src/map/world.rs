//! This module implements the [`VoxelWorld`] component and associated logic.

use bevy::prelude::*;
use bevy::utils::HashMap;

use super::chunk::ChunkData;
use super::remesh::{NeedsRemesh, UniqueBlocks};
use crate::math::{ChunkPos, Position, CHUNK_SIZE};

/// An infinite, 3D grid of voxels, represented by chunks, that make up a world.
#[derive(Debug, Default, Resource)]
pub struct VoxelWorld {
    /// The entities representing the chunks in the world.
    chunks: HashMap<ChunkPos, Entity>,
}

impl VoxelWorld {
    /// Gets the chunk entity at the given position, if it exists.
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<Entity> {
        self.chunks.get(&pos).copied()
    }
}

/// Commands for spawning and despawning chunks within a voxel world.
pub trait VoxelWorldCommands {
    /// Spawns a new chunk in the world using the provided block data.
    ///
    /// If the chunk already exists, the data within the chunk will be replaced
    /// by the newly provided data, and the provided bundle will be inserted
    /// into the existing chunk entity, overwriting any existing components.
    fn spawn_chunk(&mut self, pos: ChunkPos, data: ChunkData);

    /// Despawns the chunk at the given position within the world. This will
    /// recursively despawn all entities that are children of the chunk entity.
    ///
    /// If the chunk does not exist, this command will do nothing.
    fn despawn_chunk(&mut self, pos: ChunkPos);

    /// Despawns all chunks within the world. This will recursively despawn all
    /// entities that are children of the chunk entities as well.
    fn clear_chunks(&mut self);
}

impl<'w, 's> VoxelWorldCommands for Commands<'w, 's> {
    fn spawn_chunk(&mut self, pos: ChunkPos, data: ChunkData) {
        self.add(move |app: &mut World| {
            let world = app.get_resource::<VoxelWorld>().unwrap();

            if let Some(chunk_id) = world.get_chunk(pos) {
                let Some(mut chunk) = app.get_mut::<ChunkData>(chunk_id) else {
                    error!(
                        "VoxelWorld component contains invalid chunk entity reference {chunk_id}"
                    );
                    return;
                };

                *chunk = data;
                return;
            }

            let chunk_id = app
                .spawn((
                    Position { block: pos.into() },
                    data,
                    UniqueBlocks::default(),
                    NeedsRemesh,
                    SpatialBundle {
                        transform: Transform::from_xyz(
                            pos.x as f32 * CHUNK_SIZE as f32,
                            pos.y as f32 * CHUNK_SIZE as f32,
                            pos.z as f32 * CHUNK_SIZE as f32,
                        ),
                        ..default()
                    },
                ))
                .id();

            let mut world = app.get_resource_mut::<VoxelWorld>().unwrap();
            world.chunks.insert(pos, chunk_id);
        });
    }

    fn despawn_chunk(&mut self, pos: ChunkPos) {
        self.add(move |app: &mut World| {
            let world = app.get_resource::<VoxelWorld>().unwrap();

            if let Some(chunk_id) = world.get_chunk(pos) {
                app.entity_mut(chunk_id).despawn_recursive();

                let mut world = app.get_resource_mut::<VoxelWorld>().unwrap();
                world.chunks.remove(&pos);
            }
        });
    }

    fn clear_chunks(&mut self) {
        self.add(move |app: &mut World| {
            let mut world = app.get_resource_mut::<VoxelWorld>().unwrap();

            let mut chunks = HashMap::default();
            std::mem::swap(&mut world.chunks, &mut chunks);

            for (_, chunk_id) in chunks.iter() {
                app.entity_mut(*chunk_id).despawn_recursive();
            }
        });
    }
}
