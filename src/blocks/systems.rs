//! The various systems used within the [`BlocksPlugin`].

use bevy::gltf::{GltfMesh, GltfNode};
use bevy::math::Vec3A;
use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use uuid::Uuid;

use super::mesh::{BlockMesh, BlockVertex};
use super::model::BlockModel;
use super::occlusion::OccludedBy;
use super::shape::{BlockFace, BlockShape};
use super::tileset::{TilePos, Tileset};
use super::{AIR_BLOCK_NAME, AIR_BLOCK_UUID, Block, RenderedBlock};
use crate::blocks::mesh::BlockMeshPart;
use crate::math::{FaceDirection, FaceRotation};
use crate::utilities::meshbuf::MeshBuf;

/// This system listens for changes to [`RenderedBlock`] components and updates
/// the models to point to the correct mesh and material for the target block.
#[allow(clippy::type_complexity)]
pub fn update_rendered_block_model(
    mut meshes: ResMut<Assets<Mesh>>,
    models: Query<&BlockModel>,
    mut rendered: Query<
        (
            &RenderedBlock,
            &mut Handle<Mesh>,
            &mut Handle<StandardMaterial>,
        ),
        Changed<RenderedBlock>,
    >,
) {
    for (block, mut mesh, mut material) in rendered.iter_mut() {
        let Ok(model) = models.get(block.block) else {
            warn!("Tried to update model for RenderedBlock, but the block entity does not exist.");
            continue;
        };

        let (block_mesh, block_mat) = match model {
            BlockModel::None => {
                *mesh = Default::default();
                *material = Default::default();
                continue;
            }
            BlockModel::Primitive {
                material: block_mat,
                mesh: block_mesh,
                ..
            } => (block_mesh, block_mat),
            BlockModel::Custom {
                mesh: block_mesh,
                material: block_mat,
                ..
            } => (block_mesh, block_mat),
        };

        let mut mesh_buf = MeshBuf::new();
        block_mesh.append_to(OccludedBy::empty(), &mut mesh_buf);
        let bevy_mesh: Mesh = mesh_buf.into();

        *mesh = meshes.add(bevy_mesh);
        *material = block_mat.clone();
    }
}

/// This system listens for changes to block models and forwards the changes to
/// the rendered blocks. This system dereferences the [`RenderedBlock`]
/// component to update the block model.
pub fn forward_model_changes_to_rendered(
    models: Query<Entity, Changed<BlockModel>>,
    mut rendered: Query<&mut RenderedBlock>,
) {
    for model in models.iter() {
        for mut rendered_block in rendered.iter_mut() {
            if rendered_block.block == model {
                rendered_block.block = model;
            }
        }
    }
}

/// This system listens for changes in block shapes and updates the block models
/// accordingly.
pub fn update_block_model(
    asset_server: Res<AssetServer>,
    chunk_materials: Query<(&Handle<StandardMaterial>, &Name), With<Tileset>>,
    mut models: Query<(&mut BlockModel, &BlockShape, &Name), Changed<BlockShape>>,
) {
    for (mut model, shape, name) in models.iter_mut() {
        info!("Updating block model for block: {}", name);

        match shape {
            BlockShape::None => {
                *model = BlockModel::None;
            }
            BlockShape::Cube {
                tileset,
                top,
                bottom,
                north,
                south,
                east,
                west,
            } => {
                let material = chunk_materials
                    .iter()
                    .find(|(_, name)| ***name == *tileset)
                    .map(|(material, _)| material.clone())
                    .unwrap_or_else(|| {
                        warn!(
                            "Tried to update block model for {}, but failed to find material for tileset: {}",
                            name,
                            tileset
                        );
                        Default::default()
                    });

                let mut mesh = BlockMesh::default();

                let mut top_quad = quad(
                    FaceDirection::Up.rotation_quat(),
                    Vec3::new(0.0, 0.5, 0.0) + Vec3::splat(0.5),
                    Vec3::ONE,
                    top.tile,
                );
                update_uv(&mut top_quad, top);
                mesh.top = Some(top_quad.into());

                let mut bottom_quad = quad(
                    FaceDirection::Down.rotation_quat(),
                    Vec3::new(0.0, -0.5, 0.0) + Vec3::splat(0.5),
                    Vec3::ONE,
                    bottom.tile,
                );
                update_uv(&mut bottom_quad, bottom);
                mesh.bottom = Some(bottom_quad.into());

                let mut north_quad = quad(
                    FaceDirection::North.rotation_quat(),
                    Vec3::new(0.0, 0.0, -0.5) + Vec3::splat(0.5),
                    Vec3::ONE,
                    north.tile,
                );
                update_uv(&mut north_quad, north);
                mesh.north = Some(north_quad.into());

                let mut south_quad = quad(
                    FaceDirection::South.rotation_quat(),
                    Vec3::new(0.0, 0.0, 0.5) + Vec3::splat(0.5),
                    Vec3::ONE,
                    south.tile,
                );
                update_uv(&mut south_quad, south);
                mesh.south = Some(south_quad.into());

                let mut east_quad = quad(
                    FaceDirection::East.rotation_quat(),
                    Vec3::new(0.5, 0.0, 0.0) + Vec3::splat(0.5),
                    Vec3::ONE,
                    east.tile,
                );
                update_uv(&mut east_quad, east);
                mesh.east = Some(east_quad.into());

                let mut west_quad = quad(
                    FaceDirection::West.rotation_quat(),
                    Vec3::new(-0.5, 0.0, 0.0) + Vec3::splat(0.5),
                    Vec3::ONE,
                    west.tile,
                );
                update_uv(&mut west_quad, west);
                mesh.west = Some(west_quad.into());

                let bounds = mesh.get_bounds();

                *model = BlockModel::Primitive {
                    material,
                    mesh: Box::new(mesh),
                    bounds,
                };
            }
            BlockShape::Custom { asset } => {
                let model_path = format!("project://models/{asset}.glb");
                let default_mat = GltfAssetLabel::DefaultMaterial.from_asset(model_path.clone());

                *model = BlockModel::Custom {
                    material: asset_server.load(default_mat),
                    asset: asset_server.load(model_path),
                    bounds: Aabb3d::new(Vec3A::ZERO, Vec3A::ZERO),
                    mesh: Default::default(),
                };

                debug!("Loading custom mesh '{asset}.obj' for block: {name}");
            }
        }
    }
}

/// This system listens for asset events and updates custom block models as the
/// linked assets finish loading.
pub fn update_custom_block_model_mesh(
    mut asset_events: EventReader<AssetEvent<Gltf>>,
    gltf: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    meshes: Res<Assets<Mesh>>,
    mut models: Query<(&mut BlockModel, &Name)>,
) {
    for ev in asset_events.read() {
        let AssetEvent::LoadedWithDependencies { id } = ev else {
            continue;
        };

        info!("Loaded custom mesh asset with ID: {}", id);

        for (mut model, name) in models.iter_mut() {
            let BlockModel::Custom {
                asset,
                mesh,
                bounds,
                material,
                ..
            } = &mut *model
            else {
                continue;
            };

            if asset.id() != *id {
                continue;
            }

            let Some(gltf_data) = gltf.get(asset) else {
                error!("Failed to retrieve custom mesh for block: {name}");
                continue;
            };

            let mut block_mesh = BlockMeshPart::default();

            for gltf_node_handle in &gltf_data.nodes {
                let gltf_node = gltf_nodes.get(gltf_node_handle).unwrap();

                let mut transform = gltf_node.transform;
                transform.translation += Vec3::new(0.5, 0.0, 0.5);

                if let Some(mesh_handle) = &gltf_node.mesh {
                    let gltf_mesh = gltf_meshes.get(mesh_handle).unwrap();
                    for primitive in &gltf_mesh.primitives {
                        if let Some(mat) = &primitive.material {
                            *material = mat.clone();
                        }
                        let raw_mesh = meshes.get(&primitive.mesh).unwrap();
                        block_mesh.extend(&BlockMeshPart::new_from(raw_mesh, transform));
                    }
                }
            }

            let block_mesh = BlockMesh {
                center: Some(block_mesh),
                ..default()
            };
            *bounds = block_mesh.get_bounds();
            *mesh = Box::new(block_mesh);

            info!("Loaded custom mesh model for block: {name}");
        }
    }
}

/// Creates a quad with the given rotation, translation, and scale.
///
/// The quad, before transformation, is a unit square with the bottom-left
/// corner at `(-0.5, -0.5, 0.0)` and the top-right corner at `(0.5, 0.5, 0.0)`.
/// The quad is facing `+Z`.
fn quad(rot: Quat, translate: Vec3, scale: Vec3, tile: TilePos) -> [BlockVertex; 4] {
    let mut vertices = [BlockVertex::default(); 4];

    vertices[0].position = rot * (Vec3::new(-0.5, -0.5, 0.0) * scale) + translate;
    vertices[1].position = rot * (Vec3::new(0.5, -0.5, 0.0) * scale) + translate;
    vertices[2].position = rot * (Vec3::new(0.5, 0.5, 0.0) * scale) + translate;
    vertices[3].position = rot * (Vec3::new(-0.5, 0.5, 0.0) * scale) + translate;

    vertices[0].normal = rot * Vec3::new(0.0, 0.0, 1.0);
    vertices[1].normal = rot * Vec3::new(0.0, 0.0, 1.0);
    vertices[2].normal = rot * Vec3::new(0.0, 0.0, 1.0);
    vertices[3].normal = rot * Vec3::new(0.0, 0.0, 1.0);

    vertices[0].uv = Vec2::new(0.0, 1.0);
    vertices[1].uv = Vec2::new(1.0, 1.0);
    vertices[2].uv = Vec2::new(1.0, 0.0);
    vertices[3].uv = Vec2::new(0.0, 0.0);

    vertices[0].tile = Some(tile);
    vertices[1].tile = Some(tile);
    vertices[2].tile = Some(tile);
    vertices[3].tile = Some(tile);

    vertices
}

/// Updates the UV coordinates of a quad based on the texture properties of the
/// face.
fn update_uv(quad: &mut [BlockVertex; 4], face: &BlockFace) {
    let uv = [
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
    ];

    let uv = if face.mirror_x {
        [uv[1], uv[0], uv[3], uv[2]]
    } else {
        uv
    };

    let uv = if face.mirror_y {
        [uv[3], uv[2], uv[1], uv[0]]
    } else {
        uv
    };

    let uv = match face.rotation {
        FaceRotation::C0 => uv,
        FaceRotation::C90 => [uv[3], uv[0], uv[1], uv[2]],
        FaceRotation::C180 => [uv[2], uv[3], uv[0], uv[1]],
        FaceRotation::C270 => [uv[1], uv[2], uv[3], uv[0]],
    };

    for (vertex, uv) in quad.iter_mut().zip(uv.iter()) {
        vertex.uv = *uv;
    }
}

/// This system is called on startup to load all block definitions into the
/// world.
pub fn load_blocks(mut commands: Commands) {
    // TODO: Load blocks from a file or database.

    commands.spawn((
        Block {
            uuid: AIR_BLOCK_UUID,
        },
        Name::new(AIR_BLOCK_NAME),
        BlockModel::default(),
        BlockShape::None,
    ));

    commands.spawn((
        Block {
            uuid: Uuid::new_v4(),
        },
        Name::new("Grass"),
        BlockModel::default(),
        BlockShape::Cube {
            tileset: "overworld".to_string(),
            top: BlockFace {
                tile: TilePos::new(0, 0),
                ..default()
            },
            bottom: BlockFace {
                tile: TilePos::new(1, 0),
                ..default()
            },
            north: BlockFace {
                tile: TilePos::new(2, 0),
                ..default()
            },
            south: BlockFace {
                tile: TilePos::new(2, 0),
                ..default()
            },
            east: BlockFace {
                tile: TilePos::new(2, 0),
                ..default()
            },
            west: BlockFace {
                tile: TilePos::new(2, 0),
                ..default()
            },
        },
    ));

    commands.spawn((
        Block {
            uuid: Uuid::new_v4(),
        },
        Name::new("Dirt"),
        BlockModel::default(),
        BlockShape::Cube {
            tileset: "overworld".to_string(),
            top: BlockFace {
                tile: TilePos::new(1, 0),
                ..default()
            },
            bottom: BlockFace {
                tile: TilePos::new(1, 0),
                ..default()
            },
            north: BlockFace {
                tile: TilePos::new(1, 0),
                ..default()
            },
            south: BlockFace {
                tile: TilePos::new(1, 0),
                ..default()
            },
            east: BlockFace {
                tile: TilePos::new(1, 0),
                ..default()
            },
            west: BlockFace {
                tile: TilePos::new(1, 0),
                ..default()
            },
        },
    ));

    commands.spawn((
        Block {
            uuid: Uuid::new_v4(),
        },
        Name::new("Debug"),
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

    commands.spawn((
        Block {
            uuid: Uuid::new_v4(),
        },
        Name::new("Sign 1"),
        BlockModel::default(),
        BlockShape::Custom {
            asset: "sign1".to_string(),
        },
    ));
}
