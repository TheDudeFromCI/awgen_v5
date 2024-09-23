//! Temporary buffer for storing mesh data.

use bevy::asset::{Assets, Handle};
use bevy::prelude::{Mesh, ResMut};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::map::tileset::ATTRIBUTE_TILE_INDEX;

/// A temporary buffer for storing mesh data.
#[derive(Debug, Default, Clone)]
pub struct MeshBuf {
    /// The vertices of the mesh.
    pub positions: Vec<[f32; 3]>,

    /// The texture coordinates of the mesh.
    pub uvs: Vec<[f32; 2]>,

    /// The normals of the mesh.
    pub normals: Vec<[f32; 3]>,

    /// The layer UVs of the mesh. (Optional)
    ///
    /// If this is empty, the mesh will not have any layers. If this is not
    /// empty, the [`ATTRIBUTE_TILE_INDEX`] attribute will be added to the mesh.
    pub layers: Vec<u32>,

    /// The indices of the mesh.
    pub indices: Vec<u32>,
}

impl MeshBuf {
    /// The initial capacity of the vertices.
    pub const INIT_CAPACITY_VERTS: usize = 1024;

    /// The initial capacity of the indices.
    pub const INIT_CAPACITY_INDICES: usize = 2048;

    /// Creates a new mesh buffer.
    pub fn new() -> Self {
        Self {
            positions: Vec::with_capacity(Self::INIT_CAPACITY_VERTS),
            uvs: Vec::with_capacity(Self::INIT_CAPACITY_VERTS),
            normals: Vec::with_capacity(Self::INIT_CAPACITY_VERTS),
            layers: Vec::with_capacity(Self::INIT_CAPACITY_VERTS),
            indices: Vec::with_capacity(Self::INIT_CAPACITY_INDICES),
        }
    }

    /// Gets a reference to the vertices of the mesh.
    pub fn positions(&self) -> &[[f32; 3]] {
        &self.positions
    }

    /// Gets a reference to the indices of the mesh.
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Gets a reference to the texture coordinates of the mesh.
    pub fn tex_coords(&self) -> &[[f32; 2]] {
        &self.uvs
    }

    /// Gets a reference to the normals of the mesh.
    pub fn normals(&self) -> &[[f32; 3]] {
        &self.normals
    }

    /// Gets a reference to the layer UVs of the mesh.
    pub fn layers(&self) -> &[u32] {
        &self.layers
    }

    /// Gets the number of triangles in the mesh.
    pub fn tri_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Compiles this [`MeshBuf`] into a [`Mesh`] and updates the given mesh
    /// asset handle.
    pub fn update_handle(self, handle: &Handle<Mesh>, meshes: &mut ResMut<Assets<Mesh>>) {
        let mesh = Mesh::from(self);
        meshes.insert(handle, mesh);
    }
}

impl From<MeshBuf> for Mesh {
    fn from(value: MeshBuf) -> Self {
        let indices = if value.indices.len() > u16::MAX as usize {
            Indices::U32(value.indices)
        } else {
            Indices::U16(value.indices.iter().map(|&i| i as u16).collect())
        };

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, value.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, value.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, value.uvs);

        if !value.layers.is_empty() {
            mesh = mesh.with_inserted_attribute(ATTRIBUTE_TILE_INDEX, value.layers);
        }

        mesh = mesh.with_inserted_indices(indices);
        mesh
    }
}
