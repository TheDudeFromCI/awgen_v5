//! This module implements the components amd systems used for placing and
//! removing blocks in the world.

use bevy::prelude::*;
use bevy_mod_picking::events::{Click, Pointer};
use bevy_mod_picking::prelude::PointerButton;

use crate::blocks::params::BlockFinder;
use crate::gizmos::cursor::CursorRaycast;
use crate::map::ChunkCollider;
use crate::map::chunk::ChunkData;
use crate::map::remesh::NeedsRemesh;
use crate::map::world::{VoxelWorld, VoxelWorldCommands};
use crate::ui::hotbar::resource::{Hotbar, HotbarSlotData};

/// This system places a block at the cursor position when the left mouse button
/// is pressed.
#[allow(clippy::too_many_arguments)]
pub fn place_block(
    mut click_events: EventReader<Pointer<Click>>,
    chunk_colliders: Query<Entity, With<ChunkCollider>>,
    block_finder: BlockFinder,
    hotbar: Res<Hotbar>,
    cursor: Res<CursorRaycast>,
    world: Res<VoxelWorld>,
    mut chunks: Query<&mut ChunkData>,
    mut commands: Commands,
) {
    for ev in click_events.read() {
        if ev.button != PointerButton::Primary {
            trace!("Ignoring click event: {}; Wrong button.", ev);
            continue;
        }

        if !chunk_colliders.contains(ev.target) {
            trace!("Ignoring click event: {}; Not a chunk collider.", ev);
            continue;
        }

        let HotbarSlotData::Block(place_block) = hotbar.get_selected() else {
            trace!("Ignoring click event: {}; No block selected.", ev);
            return;
        };

        let Some(hit) = &cursor.block else {
            trace!("Ignoring click event: {}; No block hit in raycast.", ev);
            return;
        };

        let air_block = block_finder.find("air").unwrap();
        let target_pos = hit.block.shift(hit.face, 1);

        let Some(chunk_id) = world.get_chunk(target_pos.into()) else {
            trace!(
                "No chunk found at target position: {}; Creating new one.",
                target_pos
            );
            let mut new_chunk = ChunkData::fill(air_block);
            new_chunk.set(target_pos, place_block);
            commands.spawn_chunk(target_pos.into(), new_chunk);
            return;
        };

        let Ok(mut chunk) = chunks.get_mut(chunk_id) else {
            error!("Failed to get chunk data for chunk: {};", chunk_id);
            return;
        };

        chunk.set(target_pos, place_block);
        commands.entity(chunk_id).insert(NeedsRemesh);
        trace!(
            "Placed block: {:?} at position: {:?}",
            place_block, target_pos
        );
    }
}

/// This system removes a block at the cursor position when the right mouse
/// button
pub fn remove_block(
    mut click_events: EventReader<Pointer<Click>>,
    chunk_colliders: Query<Entity, With<ChunkCollider>>,
    block_finder: BlockFinder,
    cursor: Res<CursorRaycast>,
    world: Res<VoxelWorld>,
    mut chunks: Query<&mut ChunkData>,
    mut commands: Commands,
) {
    for ev in click_events.read() {
        if ev.button != PointerButton::Secondary {
            trace!("Ignoring click event: {}; Wrong button.", ev);
            continue;
        }

        if !chunk_colliders.contains(ev.target) {
            trace!("Ignoring click event: {}; Not a chunk collider.", ev);
            continue;
        }

        let Some(hit) = &cursor.block else {
            trace!("Ignoring click event: {}; No block hit in raycast.", ev);
            return;
        };

        let Some(chunk_id) = world.get_chunk(hit.block.into()) else {
            trace!(
                "No chunk found at target position: {}; Nothing to remove.",
                hit.block
            );
            return;
        };

        let Ok(mut chunk) = chunks.get_mut(chunk_id) else {
            error!("Failed to get chunk data for chunk: {}", chunk_id);
            return;
        };

        let air_block = block_finder.find("air").unwrap();

        trace!("Removing block at position: {}", hit.block);
        let dirty = chunk.set(hit.block, air_block);

        if dirty {
            if chunk.try_convert_to_single() {
                trace!("Despawning empty chunk at: {:?}", hit.block);
                commands.despawn_chunk(hit.block.into());
            } else {
                commands.entity(chunk_id).insert(NeedsRemesh);
            }
        }
    }
}
