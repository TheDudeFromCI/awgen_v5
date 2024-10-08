//! This module implements the components amd systems used for placing and
//! removing blocks in the world.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::blocks::params::BlockFinder;
use crate::gizmos::cursor::CursorRaycast;
use crate::map::chunk::ChunkData;
use crate::map::remesh::NeedsRemesh;
use crate::map::world::{VoxelWorld, VoxelWorldCommands};
use crate::ui::hotbar::resource::{Hotbar, HotbarSlotData};

/// A local timer for block placement/removal.
#[derive(Debug, SystemParam)]
pub struct PlacementTimer<'w, 's> {
    /// The current frame time.
    time: Res<'w, Time>,

    /// The number of seconds elapsed since the last block placement/removal.
    elapsed: Local<'s, f32>,

    /// The mouse button input state.
    mouse_button: Res<'w, ButtonInput<MouseButton>>,
}

impl<'w, 's> PlacementTimer<'w, 's> {
    /// The time interval between the first block placement/removal and the
    /// start of the repeating interval.
    pub const INIT_STATE: f32 = 0.05;

    /// The time interval between subsequent block placement/removal.
    pub const INTERVAL: f32 = 0.15;

    /// Checks if a block should be placed/removed on the current frame.
    ///
    /// Calling this method will update the internal state of the timer.
    pub fn check_placement(&mut self, button: MouseButton) -> bool {
        if !self.mouse_button.pressed(button) {
            return false;
        }

        if self.mouse_button.just_pressed(button) {
            *self.elapsed = 0.0;
            return true;
        }

        *self.elapsed += self.time.delta_seconds();
        if *self.elapsed >= Self::INIT_STATE + Self::INTERVAL {
            *self.elapsed = Self::INIT_STATE;
            true
        } else {
            false
        }
    }
}

/// This system places a block at the cursor position when the left mouse button
/// is pressed.
pub fn place_block(
    mut timer: PlacementTimer,
    block_finder: BlockFinder,
    hotbar: Res<Hotbar>,
    cursor: Res<CursorRaycast>,
    world: Res<VoxelWorld>,
    mut chunks: Query<&mut ChunkData>,
    mut commands: Commands,
) {
    if !timer.check_placement(MouseButton::Left) {
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
    mut timer: PlacementTimer,
    block_finder: BlockFinder,
    cursor: Res<CursorRaycast>,
    world: Res<VoxelWorld>,
    mut chunks: Query<&mut ChunkData>,
    mut commands: Commands,
) {
    if !timer.check_placement(MouseButton::Right) {
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
