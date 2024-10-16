//! This module implements the components amd systems used for placing and
//! removing blocks in the world.

use bevy::prelude::*;

use crate::blocks::params::BlockFinder;
use crate::gizmos::cursor::CursorRaycast;
use crate::map::chunk::ChunkData;
use crate::map::remesh::NeedsRemesh;
use crate::map::world::{VoxelWorld, VoxelWorldCommands};
use crate::ui::hotbar::resource::{Hotbar, HotbarSlotData};

/// This system places a block at the cursor position when the left mouse button
/// is pressed.
pub fn place_block(
    mouse_button: Res<ButtonInput<MouseButton>>,
    block_finder: BlockFinder,
    hotbar: Res<Hotbar>,
    cursor: Res<CursorRaycast>,
    world: Res<VoxelWorld>,
    mut chunks: Query<&mut ChunkData>,
    mut commands: Commands,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let HotbarSlotData::Block(place_block) = hotbar.get_selected() else {
        return;
    };

    let Some(hit) = &cursor.block else {
        return;
    };

    let air_block = block_finder.find("air").unwrap();

    let target_pos = hit.block.shift(hit.face, 1);

    let Some(chunk_id) = world.get_chunk(target_pos.into()) else {
        let mut new_chunk = ChunkData::fill(air_block);
        new_chunk.set(target_pos, place_block);
        commands.spawn_chunk(target_pos.into(), new_chunk);
        return;
    };

    let Ok(mut chunk) = chunks.get_mut(chunk_id) else {
        return;
    };

    chunk.set(target_pos, place_block);
    commands.entity(chunk_id).insert(NeedsRemesh);
}

/// This system removes a block at the cursor position when the right mouse
/// button
pub fn remove_block(
    mouse_button: Res<ButtonInput<MouseButton>>,
    block_finder: BlockFinder,
    cursor: Res<CursorRaycast>,
    world: Res<VoxelWorld>,
    mut chunks: Query<&mut ChunkData>,
    mut commands: Commands,
) {
    if !mouse_button.just_pressed(MouseButton::Right) {
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

    let air_block = block_finder.find("air").unwrap();

    let dirty = chunk.set(hit.block, air_block);
    if dirty {
        if chunk.try_convert_to_single() {
            // chunk only contains air. Despawn it.
            commands.despawn_chunk(hit.block.into());
        } else {
            commands.entity(chunk_id).insert(NeedsRemesh);
        }
    }
}