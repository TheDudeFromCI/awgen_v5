//! This module handles the implementation of the voxel world logic and
//! rendering.

use bevy::prelude::*;
use blocks::model::BlockModel;
use blocks::shape::{BlockFace, BlockShape};
use blocks::Block;
use chunk::ChunkData;
use world::{VoxelWorld, VoxelWorldCommands};

use crate::math::{BlockPos, ChunkPos, CHUNK_SIZE};
use crate::tileset::{TilePos, TilesetBundle};
use crate::ui::menu::MainMenuState;

pub mod blocks;
pub mod chunk;
pub mod editor;
pub mod remesh;
pub mod world;

/// The plugin responsible for managing the voxel world.
pub struct VoxelWorldPlugin;
impl Plugin for VoxelWorldPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_plugins((
            remesh::ChunkRemeshPlugin,
            blocks::BlocksPlugin,
            editor::MapEditorPlugin,
        ))
        .init_resource::<VoxelWorld>()
        .add_systems(OnEnter(MainMenuState::Editor), setup);
    }
}

/// Sets up the voxel world.
pub fn setup(
    mut ambient_light: ResMut<AmbientLight>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let tileset_image = asset_server.load("tilesets/overworld.png");
    commands.spawn(TilesetBundle {
        name: Name::new("overworld"),
        image: tileset_image.clone(),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(tileset_image),
            perceptual_roughness: 1.0,
            ..default()
        }),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.8, 0.0)),
        ..default()
    });
    ambient_light.brightness = 1000.0;

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
                    tile: TilePos::new(0, 0),
                    ..default()
                },
                bottom: BlockFace {
                    tile: TilePos::new(0, 0),
                    ..default()
                },
                north: BlockFace {
                    tile: TilePos::new(0, 0),
                    ..default()
                },
                south: BlockFace {
                    tile: TilePos::new(0, 0),
                    ..default()
                },
                east: BlockFace {
                    tile: TilePos::new(0, 0),
                    ..default()
                },
                west: BlockFace {
                    tile: TilePos::new(0, 0),
                    ..default()
                },
            },
        ))
        .id();

    commands.spawn((
        Block,
        Name::new("debug"),
        BlockModel::default(),
        BlockShape::Cube {
            tileset: "overworld".to_string(),
            top: BlockFace {
                tile: TilePos::new(2, 1),
                ..default()
            },
            bottom: BlockFace {
                tile: TilePos::new(3, 1),
                ..default()
            },
            north: BlockFace {
                tile: TilePos::new(0, 1),
                ..default()
            },
            south: BlockFace {
                tile: TilePos::new(1, 1),
                ..default()
            },
            east: BlockFace {
                tile: TilePos::new(4, 1),
                ..default()
            },
            west: BlockFace {
                tile: TilePos::new(5, 1),
                ..default()
            },
        },
    ));

    let mut chunk_data = ChunkData::fill(air);
    for x in 0 .. CHUNK_SIZE {
        for z in 0 .. CHUNK_SIZE {
            chunk_data.set(BlockPos::new(x as i32, 0, z as i32), grass);
        }
    }

    commands.spawn_chunk(ChunkPos::new(0, 0, 0), chunk_data);
}
