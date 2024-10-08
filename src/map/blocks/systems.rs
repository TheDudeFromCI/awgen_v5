//! The various systems used within the [`BlocksPlugin`].

use bevy::math::bounding::Aabb3d;
use bevy::math::Vec3A;
use bevy::prelude::*;

use super::mesh::BlockVertex;
use super::model::BlockModel;
use super::occlusion::OccludedBy;
use super::shape::{BlockFace, BlockShape};
use super::RenderedBlock;
use crate::map::blocks::mesh::BlockMesh;
use crate::math::{FaceDirection, FaceRotation};
use crate::tileset::{TilePos, Tileset};
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

        match model {
            BlockModel::None => {
                *mesh = Default::default();
                *material = Default::default();
            }
            BlockModel::Primitive {
                material: block_mat,
                mesh: block_mesh,
                ..
            } => {
                let mut mesh_buf = MeshBuf::new();
                block_mesh.append_to(OccludedBy::empty(), &mut mesh_buf);
                let bevy_mesh: Mesh = mesh_buf.into();

                *mesh = meshes.add(bevy_mesh);
                *material = block_mat.clone();
            }
            BlockModel::Custom {
                material: block_mat,
                mesh: block_mesh,
                ..
            } => {
                *mesh = block_mesh.clone();
                *material = block_mat.clone();
            }
        }
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
                *model = BlockModel::Custom {
                    material: asset_server.load(
                        GltfAssetLabel::Material {
                            index: 0,
                            is_scale_inverted: false,
                        }
                        .from_asset(asset.clone()),
                    ),
                    mesh: asset_server.load(GltfAssetLabel::Mesh(0).from_asset(asset.clone())),

                    // TODO: Update bounds when mesh loading is complete.
                    bounds: Aabb3d::new(Vec3A::ZERO, Vec3A::ZERO),
                };
            }
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

    vertices[0].tile = tile;
    vertices[1].tile = tile;
    vertices[2].tile = tile;
    vertices[3].tile = tile;

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
