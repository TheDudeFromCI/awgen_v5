//! This module implements the systems used within the map editor plugin.

use bevy::prelude::*;

use crate::gizmos::cursor::CursorRaycast;
use crate::map::blocks::Block;
use crate::map::chunk::ChunkData;
use crate::map::remesh::NeedsRemesh;
use crate::map::world::VoxelWorld;

/// This system places a block at the cursor position when the left mouse button
/// is pressed.
pub fn place_block(
    cursor: Res<CursorRaycast>,
    input: Res<ButtonInput<MouseButton>>,
    world: Res<VoxelWorld>,
    blocks: Query<(Entity, &Name), With<Block>>,
    mut chunks: Query<&mut ChunkData>,
    mut commands: Commands,
) {
    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(hit) = &cursor.block else {
        return;
    };

    let target_pos = hit.block.shift(hit.face, 1);

    let Some(chunk_id) = world.get_chunk(target_pos.into()) else {
        return;
    };

    let Ok(mut chunk) = chunks.get_mut(chunk_id) else {
        return;
    };

    let grass_name: Name = "grass".into();
    let Some(grass_id) = blocks
        .iter()
        .find(|(_, name)| **name == grass_name)
        .map(|(entity, _)| entity)
    else {
        return;
    };

    chunk.set(target_pos, grass_id);
    commands.entity(chunk_id).insert(NeedsRemesh);
}

/// This system removes a block at the cursor position when the right mouse
/// button
pub fn remove_block(
    cursor: Res<CursorRaycast>,
    input: Res<ButtonInput<MouseButton>>,
    world: Res<VoxelWorld>,
    blocks: Query<(Entity, &Name), With<Block>>,
    mut chunks: Query<&mut ChunkData>,
    mut commands: Commands,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(hit) = &cursor.block else {
        return;
    };

    let Some(chunk_id) = world.get_chunk(hit.block.into()) else {
        return;
    };

    let Ok(mut chunk) = chunks.get_mut(chunk_id) else {
        return;
    };

    let air_name: Name = "air".into();
    let Some(air_id) = blocks
        .iter()
        .find(|(_, name)| **name == air_name)
        .map(|(entity, _)| entity)
    else {
        return;
    };

    chunk.set(hit.block, air_id);
    commands.entity(chunk_id).insert(NeedsRemesh);
}
