//! Data structures for storing block meshes.

use bevy::math::Vec3A;
use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use tinyvec::TinyVec;

use super::occlusion::OccludedBy;
use super::tileset::TilePos;
use crate::utilities::meshbuf::MeshBuf;

/// The maximum number of vertices to store on the stack in a [`BlockMesh`].
const MAX_STACK_VERTICES: usize = 8;

/// The maximum number of indices to store on the stack in a [`BlockMesh`].
const MAX_STACK_INDICES: usize = 16;

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

impl BlockMesh {
    /// Returns a reference to the parts of the block mesh.
    fn parts(&self) -> [&Option<BlockMeshPart>; 7] {
        [
            &self.center,
            &self.top,
            &self.bottom,
            &self.north,
            &self.south,
            &self.east,
            &self.west,
        ]
    }

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

    /// Gets the bounds of the block mesh.
    pub fn get_bounds(&self) -> Aabb3d {
        Aabb3d::from_point_cloud(
            Vec3A::ZERO,
            Quat::IDENTITY,
            self.parts().into_iter().flatten().flat_map(|part| {
                part.vertices
                    .iter()
                    .map(|vertex| Vec3A::from(vertex.position))
            }),
        )
    }
}

impl From<&Mesh> for BlockMesh {
    fn from(value: &Mesh) -> Self {
        let mut mesh_part = BlockMeshPart::default();

        let positions = value.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let normals = value.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap();
        let uvs = value.attribute(Mesh::ATTRIBUTE_UV_0).unwrap();
        let indices = value.indices().unwrap();

        let VertexAttributeValues::Float32x3(positions) = positions else {
            panic!("Invalid position attribute");
        };

        let VertexAttributeValues::Float32x3(normals) = normals else {
            panic!("Invalid normal attribute");
        };

        let VertexAttributeValues::Float32x2(uvs) = uvs else {
            panic!("Invalid uv attribute");
        };

        for i in 0 .. positions.len() {
            mesh_part.vertices.push(BlockVertex {
                position: positions[i].into(),
                normal: normals[i].into(),
                uv: uvs[i].into(),
                tile: None,
            });
        }

        for index in indices.iter() {
            mesh_part.indices.push(index as u16);
        }

        BlockMesh {
            center: Some(mesh_part),
            ..default()
        }
    }
}

/// The mesh of a primitive block model.
#[derive(Debug, Default, Clone)]
pub struct BlockMeshPart {
    /// The vertices of the block.
    pub vertices: TinyVec<[BlockVertex; MAX_STACK_VERTICES]>,

    /// The indices of the block.
    pub indices: TinyVec<[u16; MAX_STACK_INDICES]>,
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

            if let Some(tile) = vertex.tile {
                let uv = tile.transform_uv(vertex.uv);
                mesh.uvs.push(uv.into());
            } else {
                mesh.uvs.push(vertex.uv.into());
            }
        }

        for index in self.indices.iter() {
            mesh.indices.push(*index as u32 + offset);
        }
    }

    /// Creates a block mesh part from the given mesh.
    pub fn new_from(value: &Mesh, transform: Transform) -> Self {
        let mut mesh_part = BlockMeshPart::default();
        let matrix = transform.compute_matrix();

        let positions = value.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let normals = value.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap();
        let uvs = value.attribute(Mesh::ATTRIBUTE_UV_0).unwrap();
        let indices = value.indices().unwrap();

        let VertexAttributeValues::Float32x3(positions) = positions else {
            panic!("Invalid position attribute");
        };

        let VertexAttributeValues::Float32x3(normals) = normals else {
            panic!("Invalid normal attribute");
        };

        let VertexAttributeValues::Float32x2(uvs) = uvs else {
            panic!("Invalid uv attribute");
        };

        for i in 0 .. positions.len() {
            let position = matrix * Vec3::from(positions[i]).extend(1.0);
            let normal = matrix * Vec3::from(normals[i]).extend(0.0);

            mesh_part.vertices.push(BlockVertex {
                position: position.xyz(),
                normal: normal.xyz(),
                uv: uvs[i].into(),
                tile: None,
            });
        }

        for index in indices.iter() {
            mesh_part.indices.push(index as u16);
        }

        mesh_part
    }

    /// Extends this block mesh part with the vertices and indices of another
    /// block mesh part.
    pub fn extend(&mut self, other: &BlockMeshPart) {
        let offset = self.vertices.len() as u16;

        for vertex in other.vertices.iter() {
            self.vertices.push(*vertex);
        }

        for index in other.indices.iter() {
            self.indices.push(*index + offset);
        }
    }
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

    /// The tileset position of the vertex. If set to `None`, the texture
    /// coordinates specified in the UV field will not be modified.
    pub tile: Option<TilePos>,
}
