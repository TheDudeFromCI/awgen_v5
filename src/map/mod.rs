//! This module handles the implementation of the voxel world logic and
//! rendering.

use bevy::prelude::*;
use blocks::model::{BlockFace, BlockModel, BlockShape, RenderedBlock};
use blocks::{Block, Tileset};
use chunk::ChunkData;
use pos::{BlockPos, ChunkPos, Position, CHUNK_SIZE};
use world::{VoxelWorld, VoxelWorldCommands};

use crate::ui::menu::MainMenuState;

pub mod blocks;
pub mod chunk;
pub mod pos;
pub mod world;

/// The plugin responsible for managing the voxel world.
pub struct VoxelWorldPlugin;
impl Plugin for VoxelWorldPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(OnEnter(MainMenuState::Editor), setup)
            .add_systems(
                Update,
                (
                    blocks::model::update_block_model,
                    blocks::model::forward_model_changes_to_rendered,
                    blocks::model::update_rendered_block_model,
                )
                    .chain(),
            );
    }
}

/// Sets up the voxel world.
fn setup(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    commands.spawn((
        Tileset,
        Name::new("overworld"),
        materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("tilesets/overworld.png")),
            ..default()
        }),
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    let air = commands
        .spawn((
            Block,
            Name::new("air"),
            BlockModel::default(),
            BlockShape::None,
        ))
        .id();

    let grass = commands
        .spawn((
            Block,
            Name::new("grass"),
            BlockModel::default(),
            BlockShape::Cube {
                tileset: "overworld".to_string(),
                top: BlockFace {
                    tile_index: 0,
                    ..default()
                },
                bottom: BlockFace {
                    tile_index: 0,
                    ..default()
                },
                north: BlockFace {
                    tile_index: 0,
                    ..default()
                },
                south: BlockFace {
                    tile_index: 0,
                    ..default()
                },
                east: BlockFace {
                    tile_index: 0,
                    ..default()
                },
                west: BlockFace {
                    tile_index: 0,
                    ..default()
                },
            },
        ))
        .id();

    let world_id = commands
        .spawn((VoxelWorld::default(), SpatialBundle::default()))
        .id();

    let mut chunk_data = ChunkData::fill(air);
    for x in 0 .. CHUNK_SIZE {
        for z in 0 .. CHUNK_SIZE {
            chunk_data.set(BlockPos::new(x as i32, 0, z as i32), grass);
        }
    }

    commands.spawn_chunk(
        Position {
            world: world_id,
            block: ChunkPos::new(0, 0, 0).into(),
        },
        chunk_data,
    );

    commands.spawn((RenderedBlock { block: grass }, PbrBundle::default()));
}