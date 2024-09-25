//! This module implements the [`VoxelWorld`] component and associated logic.

use bevy::prelude::*;
use bevy::utils::HashMap;

use super::chunk::ChunkData;
use super::pos::{ChunkPos, Position, CHUNK_SIZE};
use super::remesh::{NeedsRemesh, UniqueBlocks};

/// An infinite, 3D grid of voxels, represented by chunks, that make up a world.
#[derive(Debug, Default, Component)]
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
    /// Spawns a new chunk in the given world using the provided block data. The
    /// new chunk will spawn as a child of the world entity.
    ///
    /// If the chunk already exists, the data within the chunk will be replaced
    /// by the newly provided data, and the provided bundle will be inserted
    /// into the existing chunk entity, overwriting any existing components.
    ///
    /// If the world entity has a [`Transform`] component attached, the chunk
    /// will be spawned with a [`SpatialBundle`] that positions the chunk at the
    /// correct location within the world.
    ///
    /// A bundle can be provided to add additional components to the chunk
    /// entity if it is successfully spawned. If the chunk fails to spawn, the
    /// bundle will not be applied. `()` can be used to indicate no additional
    /// components are needed.
    fn spawn_chunk(&mut self, pos: Position, data: ChunkData);

    /// Despawns the chunk at the given position within the world. This will
    /// recursively despawn all entities that are children of the chunk entity.
    ///
    /// If the chunk does not exist, this command will do nothing.
    fn despawn_chunk(&mut self, pos: Position);
}

impl<'w, 's> VoxelWorldCommands for Commands<'w, 's> {
    fn spawn_chunk(&mut self, pos: Position, data: ChunkData) {
        self.add(move |app: &mut World| {
            let Some(world) = app.get::<VoxelWorld>(pos.world) else {
                error!(
                    "Failed to get VoxelWorld component for entity {}",
                    pos.world
                );
                return;
            };

            if let Some(chunk_id) = world.get_chunk(pos.block.into()) {
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
                    pos.clone(),
                    data,
                    UniqueBlocks::default(),
                    NeedsRemesh,
                    SpatialBundle {
                        transform: Transform::from_xyz(
                            pos.block.x as f32 * CHUNK_SIZE as f32,
                            pos.block.y as f32 * CHUNK_SIZE as f32,
                            pos.block.z as f32 * CHUNK_SIZE as f32,
                        ),
                        ..default()
                    },
                ))
                .id();

            let mut world = app.get_mut::<VoxelWorld>(pos.world).unwrap();
            world.chunks.insert(pos.block.into(), chunk_id);

            app.entity_mut(chunk_id).set_parent(pos.world);
        });
    }

    fn despawn_chunk(&mut self, pos: Position) {
        self.add(move |app: &mut World| {
            let Some(world) = app.get::<VoxelWorld>(pos.world) else {
                error!(
                    "Failed to get VoxelWorld component for entity {}",
                    pos.world
                );
                return;
            };

            let chunk_pos: ChunkPos = pos.block.into();

            if let Some(chunk_id) = world.get_chunk(chunk_pos) {
                app.entity_mut(chunk_id).despawn_recursive();

                let mut world = app.get_mut::<VoxelWorld>(pos.world).unwrap();
                world.chunks.remove(&chunk_pos);
            }
        });
    }
}
