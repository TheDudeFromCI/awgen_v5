//! This module handles the startup procedure for the map editor. This
//! functionality encompasses functions that are called when the user enters the
//! map editor.

use bevy::prelude::*;

use crate::blocks::params::BlockFinder;
use crate::map::chunk::ChunkData;
use crate::map::world::VoxelWorldCommands;
use crate::math::{BlockPos, CHUNK_SIZE, ChunkPos};
use crate::ui::hotbar::resource::{Hotbar, HotbarSlotData};

/// This system is called when the application enters the map editor. It sets up
/// the map editor environment.
pub fn prepare_map_editor(
    mut hotbar: ResMut<Hotbar>,
    block_finder: BlockFinder,
    mut ambient_light: ResMut<AmbientLight>,
    mut commands: Commands,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 4000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.8, 0.0)),
        ..default()
    });
    ambient_light.brightness = 1000.0;

    let air = block_finder.find("air").unwrap();
    let grass = block_finder.find("grass").unwrap();
    let dirt = block_finder.find("dirt").unwrap();
    let debug = block_finder.find("debug").unwrap();
    let sign1 = block_finder.find("sign1").unwrap();

    let mut chunk_data = ChunkData::fill(air);
    for x in 0 .. CHUNK_SIZE {
        for z in 0 .. CHUNK_SIZE {
            chunk_data.set(BlockPos::new(x as i32, 0, z as i32), grass);
        }
    }

    commands.spawn_chunk(ChunkPos::new(0, 0, 0), chunk_data);

    hotbar.set_slot(0, HotbarSlotData::Block(grass));
    hotbar.set_slot(1, HotbarSlotData::Block(dirt));
    hotbar.set_slot(2, HotbarSlotData::Block(debug));
    hotbar.set_slot(3, HotbarSlotData::Block(sign1));
}
