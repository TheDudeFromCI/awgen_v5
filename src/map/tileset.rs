//! This module implements tileset loading and management.

use bevy::asset::LoadState;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef};
use bevy::render::render_resource::{
    AsBindGroup,
    RenderPipelineDescriptor,
    ShaderRef,
    SpecializedMeshPipelineError,
    VertexFormat,
};
use bevy::render::texture::ImageSampler;

/// The total number of tiles in a tileset, stacked vertically.
pub const TILESET_LENGTH: usize = 256;

/// The path to the tileset shader.
pub const SHADER_PATH: &str = "awgen/shaders/tileset.wgsl";

/// The tile index attribute used by the tileset shader.
pub const ATTRIBUTE_TILE_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("TileIndex", 710310471, VertexFormat::Uint32);

/// This component is added to a tileset to indicate it's current load state.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum TilesetLoadState {
    /// The tileset is still loading.
    #[default]
    Loading,

    /// The tileset has been loaded.
    Loaded,
}

/// A marker component that defines an entity as a tileset definition.
#[derive(Debug, Default, Component)]
pub struct Tileset;

/// A bundle that defines the components of a tileset.
#[derive(Debug, Default, Bundle)]
pub struct TilesetBundle {
    /// A marker component that defines an entity as a tileset definition.
    pub tileset: Tileset,

    /// The name of the tileset.
    pub name: Name,

    /// The current load state of the tileset.
    pub load_state: TilesetLoadState,

    /// The tileset image handle.
    ///
    /// Ideally, this should be a weak handle to prevent the image from
    /// remaining in RAM after it has been loaded.
    pub image: Handle<Image>,

    /// The material used to render the tileset.
    pub material: Handle<TilesetMaterial>,
}

/// A material that is used to render a tileset.
#[derive(Debug, Default, Clone, Asset, AsBindGroup, TypePath)]
pub struct TilesetMaterial {
    /// The tileset texture of the material.
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,

    /// The alpha mode of the material.
    pub alpha_mode: AlphaMode,
}

impl Material for TilesetMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_TILE_INDEX.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

/// This system will listen for tilesets that are still loading and will finish
/// loading them if they are ready, updating their load state and rearranging
/// them into a 2D texture array.
#[allow(clippy::type_complexity)]
pub fn finish_loading_tilesets(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TilesetMaterial>>,
    mut tilesets: Query<
        (
            &Handle<Image>,
            &Handle<TilesetMaterial>,
            &mut TilesetLoadState,
            &Name,
        ),
        With<Tileset>,
    >,
) {
    for (image_handle, material_handle, mut load_state, name) in tilesets.iter_mut() {
        // Compare here `as_ref()` to avoid a mutable dereference and accidentally
        // triggering the component to be marked as dirty.
        if load_state.as_ref() == &TilesetLoadState::Loaded {
            continue;
        }

        if asset_server.load_state(image_handle.id()) != LoadState::Loaded {
            continue;
        }

        *load_state = TilesetLoadState::Loaded;

        let image = images.get_mut(image_handle).unwrap();
        image.sampler = ImageSampler::nearest();

        let mut alpha_mask = 0x00;
        image.data.iter().step_by(4).skip(3).for_each(|&alpha| {
            if alpha < 255 {
                if alpha > 0 {
                    alpha_mask |= 0x02;
                } else {
                    alpha_mask |= 0x01;
                }
            }
        });
        let alpha_mode = match alpha_mask {
            0x00 => AlphaMode::Opaque,
            0x01 => AlphaMode::Mask(0.5),
            _ => AlphaMode::Blend,
        };

        image.reinterpret_stacked_2d_as_array(TILESET_LENGTH as u32);

        let material = materials.get_mut(material_handle).unwrap();
        material.texture = Some(image_handle.clone());
        material.alpha_mode = alpha_mode;

        info!("Tileset fully loaded: {}", name);
    }
}
