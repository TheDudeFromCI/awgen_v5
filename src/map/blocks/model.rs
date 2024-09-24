//! This module implements the handling for construction of block models.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use tinyvec::TinyVec;

use super::occlusion::{OccludedBy, Occludes};
use crate::map::pos::FaceDirection;
use crate::tileset::{Tileset, TilesetMaterial};
use crate::utilities::meshbuf::MeshBuf;

/// The maximum number of vertices to store on the stack in a [`BlockMesh`].
const MAX_STACK_VERTICES: usize = 8;

/// The maximum number of indices to store on the stack in a [`BlockMesh`].
const MAX_STACK_INDICES: usize = 16;

/// The model definition of a block, as defined by the block's mesh and
/// material.
#[derive(Debug, Default, Clone, Component)]
pub enum BlockModel {
    /// The block has no model.
    #[default]
    None,

    /// The block has a primitive shape and can be used in the construction of
    /// static chunk meshes.
    Primitive {
        /// The material of the block.
        material: Handle<TilesetMaterial>,

        /// The mesh of the block.
        mesh: Box<BlockMesh>,
    },

    /// The block has a custom model and is added as a child of a chunk entity.
    Custom {
        /// The material of the block.
        material: Handle<StandardMaterial>,
        /// The mesh of the block.
        mesh: Handle<Mesh>,
    },
}

/// The shape constructor of a block.
#[derive(Debug, Default, Clone, Component, Serialize, Deserialize)]
pub enum BlockShape {
    /// No model.
    #[default]
    None,

    /// A standard cubic block.
    Cube {
        /// The tileset of the block.
        tileset: String,

        /// The texture properties of the top face of the block.
        top: BlockFace,

        /// The texture properties of the bottom face of the block.
        bottom: BlockFace,

        /// The texture properties of the north face of the block.
        north: BlockFace,

        /// The texture properties of the south face of the block.
        south: BlockFace,

        /// The texture properties of the east face of the block.
        east: BlockFace,

        /// The texture properties of the west face of the block.
        west: BlockFace,
    },

    /// A block with a custom shape.
    Custom {
        /// The asset path of the block model.
        asset: String,
    },
}

/// The texture properties of a face of a block.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BlockFace {
    /// The index of the texture within the texture atlas.
    pub tile_index: u32,

    /// The rotation of the texture.
    pub rotation: FaceRotation,

    /// Whether the texture is mirrored along the x-axis. (Before rotation)
    pub mirror_x: bool,

    /// Whether the texture is mirrored along the y-axis. (Before rotation)
    pub mirror_y: bool,
}

/// The texture rotation of a face of a block.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaceRotation {
    /// No rotation.
    #[default]
    C0,

    /// 90 degrees clockwise rotation.
    C90,

    /// 180 degrees clockwise rotation.
    C180,

    /// 270 degrees clockwise rotation.
    C270,
}

/// A vertex used within a block model.
#[derive(Debug, Default, Clone, Copy)]
pub struct BlockVertex {
    /// The position of the vertex.
    pub position: Vec3,

    /// The normal of the vertex.
    pub normal: Vec3,

    /// The UV coordinates of the vertex.
    pub uv: Vec2,

    /// The layer of the vertex.
    pub layer: u32,
}

/// The mesh of a primitive block model.
#[derive(Debug, Default, Clone)]
pub struct BlockMeshPart {
    /// The vertices of the block.
    pub vertices: TinyVec<[BlockVertex; MAX_STACK_VERTICES]>,

    /// The indices of the block.
    pub indices: TinyVec<[u16; MAX_STACK_INDICES]>,
}

/// The mesh of a primitive block model.
#[derive(Debug, Default, Clone)]
pub struct BlockMesh {
    /// The center of the block mesh. Visible if there are any faces that are
    /// not covered by other blocks.
    pub center: Option<BlockMeshPart>,

    /// The top face of the block mesh. Visible if there is no block above the
    /// block.
    pub top: Option<BlockMeshPart>,

    /// The bottom face of the block mesh. Visible if there is no block below
    /// the block.
    pub bottom: Option<BlockMeshPart>,

    /// The north face of the block mesh. Visible if there is no block to the
    /// north of the block.
    pub north: Option<BlockMeshPart>,

    /// The south face of the block mesh. Visible if there is no block to the
    /// south of the block.
    pub south: Option<BlockMeshPart>,

    /// The east face of the block mesh. Visible if there is no block to the
    /// east of the block.
    pub east: Option<BlockMeshPart>,

    /// The west face of the block mesh. Visible if there is no block to the
    /// west of the block.
    pub west: Option<BlockMeshPart>,
}

/// Creates a quad with the given rotation, translation, and scale.
///
/// The quad, before transformation, is a unit square with the bottom-left
/// corner at `(-0.5, -0.5, 0.0)` and the top-right corner at `(0.5, 0.5, 0.0)`.
/// The quad is facing `+Z`.
fn quad(rot: Quat, translate: Vec3, scale: Vec3, layer: u32) -> [BlockVertex; 4] {
    let mut vertices = [BlockVertex::default(); 4];

    vertices[0].position = rot * (Vec3::new(-0.5, -0.5, 0.0) * scale) + translate;
    vertices[1].position = rot * (Vec3::new(0.5, -0.5, 0.0) * scale) + translate;
    vertices[2].position = rot * (Vec3::new(0.5, 0.5, 0.0) * scale) + translate;
    vertices[3].position = rot * (Vec3::new(-0.5, 0.5, 0.0) * scale) + translate;

    vertices[0].normal = rot * Vec3::new(0.0, 0.0, 1.0);
    vertices[1].normal = rot * Vec3::new(0.0, 0.0, 1.0);
    vertices[2].normal = rot * Vec3::new(0.0, 0.0, 1.0);
    vertices[3].normal = rot * Vec3::new(0.0, 0.0, 1.0);

    vertices[0].uv = Vec2::new(0.0, 0.0);
    vertices[1].uv = Vec2::new(1.0, 0.0);
    vertices[2].uv = Vec2::new(1.0, 1.0);
    vertices[3].uv = Vec2::new(0.0, 1.0);

    vertices[0].layer = layer;
    vertices[1].layer = layer;
    vertices[2].layer = layer;
    vertices[3].layer = layer;

    vertices
}

/// Updates the UV coordinates of a quad based on the texture properties of the
/// face.
fn update_uv(quad: &mut [BlockVertex; 4], face: &BlockFace) {
    let uv = [
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
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

impl From<[BlockVertex; 4]> for BlockMeshPart {
    fn from(value: [BlockVertex; 4]) -> Self {
        let mut vertices = TinyVec::with_capacity(4);
        vertices.extend_from_slice(&value);

        let mut indices = TinyVec::with_capacity(6);
        indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);

        Self { vertices, indices }
    }
}

impl BlockMeshPart {
    /// Appends the vertices and indices of this block mesh part to the given
    /// mesh buffer.
    pub fn append_to(&self, mesh: &mut MeshBuf) {
        let offset = mesh.positions.len() as u32;

        for vertex in self.vertices.iter() {
            mesh.positions.push(vertex.position.into());
            mesh.normals.push(vertex.normal.into());
            mesh.uvs.push(vertex.uv.into());
            mesh.layers.push(vertex.layer as u32);
        }

        for index in self.indices.iter() {
            mesh.indices.push(*index as u32 + offset);
        }
    }
}

impl BlockMesh {
    /// Returns a mutable reference to the parts of the block mesh.
    fn parts_mut(&mut self) -> [&mut Option<BlockMeshPart>; 7] {
        [
            &mut self.center,
            &mut self.top,
            &mut self.bottom,
            &mut self.north,
            &mut self.south,
            &mut self.east,
            &mut self.west,
        ]
    }

    /// Rotates the block mesh by the given rotation.
    pub fn rotate(&mut self, rot: Quat) {
        for part in self.parts_mut().into_iter().flatten() {
            for vertex in part.vertices.iter_mut() {
                vertex.position = rot * vertex.position;
                vertex.normal = rot * vertex.normal;
            }
        }
    }

    /// Translates the block mesh by the given translation.
    pub fn translate(&mut self, translate: Vec3) {
        for part in self.parts_mut().into_iter().flatten() {
            for vertex in part.vertices.iter_mut() {
                vertex.position += translate;
            }
        }
    }

    /// Appends this block mesh to the given mesh buffer based on the provided
    /// occlusion data.
    pub fn append_to(&self, occlusion: OccludedBy, mesh: &mut MeshBuf) {
        if let Some(part) = &self.center {
            if !occlusion.is_all() {
                part.append_to(mesh);
            }
        }

        if let Some(part) = &self.top {
            if !occlusion.contains(OccludedBy::Up) {
                part.append_to(mesh);
            }
        }

        if let Some(part) = &self.bottom {
            if !occlusion.contains(OccludedBy::Down) {
                part.append_to(mesh);
            }
        }

        if let Some(part) = &self.north {
            if !occlusion.contains(OccludedBy::North) {
                part.append_to(mesh);
            }
        }

        if let Some(part) = &self.south {
            if !occlusion.contains(OccludedBy::South) {
                part.append_to(mesh);
            }
        }

        if let Some(part) = &self.east {
            if !occlusion.contains(OccludedBy::East) {
                part.append_to(mesh);
            }
        }

        if let Some(part) = &self.west {
            if !occlusion.contains(OccludedBy::West) {
                part.append_to(mesh);
            }
        }
    }
}

impl BlockShape {
    /// Gets what surrounding blocks are occluded by this block. Note that this
    /// method does not check tileset transparency and assumes that the block
    /// model is always is opaque. A tileset that contains transparent textures
    /// should always be considered as never occluding.
    ///
    /// This method also assumes that all custom models as fully transparent.
    #[inline(always)]
    pub fn occlusion(&self) -> Occludes {
        match self {
            BlockShape::None => Occludes::empty(),
            BlockShape::Cube { .. } => Occludes::all(),
            BlockShape::Custom { .. } => Occludes::empty(),
        }
    }
}

/// This component can be used to indicate a standalone [`PbrBundle`] entity
/// that reads model data from a block entity.
#[derive(Debug, Component)]
pub struct RenderedBlock {
    /// The block entity to read model data from.
    pub block: Entity,
}

/// This system listens for changes to [`RenderedBlock`] components and updates
/// the models to point to the correct mesh and material for the target block.
#[allow(clippy::type_complexity)]
pub fn update_rendered_block_model(
    mut meshes: ResMut<Assets<Mesh>>,
    models: Query<&BlockModel>,
    mut rendered: Query<(Entity, &RenderedBlock), Changed<RenderedBlock>>,
    mut commands: Commands,
) {
    for (entity, block) in rendered.iter_mut() {
        let Ok(model) = models.get(block.block) else {
            warn!("Tried to update model for RenderedBlock, but the block entity does not exist.");
            continue;
        };

        match model {
            BlockModel::None => {
                commands
                    .entity(entity)
                    .remove::<Handle<StandardMaterial>>()
                    .remove::<Handle<TilesetMaterial>>()
                    .remove::<Handle<Mesh>>();
            }
            BlockModel::Primitive {
                material: block_mat,
                mesh: block_mesh,
            } => {
                let mut mesh_buf = MeshBuf::new();
                block_mesh.append_to(OccludedBy::empty(), &mut mesh_buf);
                let bevy_mesh: Mesh = mesh_buf.into();

                commands
                    .entity(entity)
                    .remove::<Handle<StandardMaterial>>()
                    .insert((block_mat.clone(), meshes.add(bevy_mesh)));
            }
            BlockModel::Custom {
                material: block_mat,
                mesh: block_mesh,
            } => {
                commands
                    .entity(entity)
                    .remove::<Handle<TilesetMaterial>>()
                    .insert((block_mat.clone(), block_mesh.clone()));
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
    chunk_materials: Query<(&Handle<TilesetMaterial>, &Name), With<Tileset>>,
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
                    top.tile_index,
                );
                update_uv(&mut top_quad, top);
                mesh.top = Some(top_quad.into());

                let mut bottom_quad = quad(
                    FaceDirection::Down.rotation_quat(),
                    Vec3::new(0.0, -0.5, 0.0) + Vec3::splat(0.5),
                    Vec3::ONE,
                    bottom.tile_index,
                );
                update_uv(&mut bottom_quad, bottom);
                mesh.bottom = Some(bottom_quad.into());

                let mut north_quad = quad(
                    FaceDirection::North.rotation_quat(),
                    Vec3::new(0.0, 0.0, -0.5) + Vec3::splat(0.5),
                    Vec3::ONE,
                    north.tile_index,
                );
                update_uv(&mut north_quad, north);
                mesh.north = Some(north_quad.into());

                let mut south_quad = quad(
                    FaceDirection::South.rotation_quat(),
                    Vec3::new(0.0, 0.0, 0.5) + Vec3::splat(0.5),
                    Vec3::ONE,
                    south.tile_index,
                );
                update_uv(&mut south_quad, south);
                mesh.south = Some(south_quad.into());

                let mut east_quad = quad(
                    FaceDirection::East.rotation_quat(),
                    Vec3::new(0.5, 0.0, 0.0) + Vec3::splat(0.5),
                    Vec3::ONE,
                    east.tile_index,
                );
                update_uv(&mut east_quad, east);
                mesh.east = Some(east_quad.into());

                let mut west_quad = quad(
                    FaceDirection::West.rotation_quat(),
                    Vec3::new(-0.5, 0.0, 0.0) + Vec3::splat(0.5),
                    Vec3::ONE,
                    west.tile_index,
                );
                update_uv(&mut west_quad, west);
                mesh.west = Some(west_quad.into());

                *model = BlockModel::Primitive {
                    material,
                    mesh: Box::new(mesh),
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
                };
            }
        }
    }
}
