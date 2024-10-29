//! This module implements the block preview widget for the Block Editor UI
//! screen.

use bevy::prelude::*;
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::render::render_resource::{
    Extent3d,
    TextureDescriptor,
    TextureDimension,
    TextureFormat,
    TextureUsages,
};
use bevy::render::view::RenderLayers;
use bevy_egui::EguiUserTextures;

use crate::blocks::RenderedBlock;
use crate::blocks::params::BlockFinder;

/// The default image size for the block preview widget in the Block Editor UI.
pub const BLOCK_PREVIEW_SIZE: u32 = 128;

/// The scale factor used to render block previews in the Block Editor UI. A
/// scale factor of 1.0 indicates that the block preview will exactly large
/// enough to fit a block at a isometric angle. A value greater than 1.0 will
/// add a percentage of padding around the block preview.
pub const BLOCK_PREVIEW_SCALE: f32 = 1.1;

/// This is a marker component used to indicate that the entity is used to
/// render block previews in the Block Editor UI.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct BlockPreviewElement;

/// This resource contains information about the block preview widget used in
/// the Block Editor UI screen. This resource may not exist when the game is not
/// in the Editor game state.
#[derive(Debug, Resource)]
pub struct BlockPreviewWidget {
    /// The image handle of the target render texture.
    pub handle: Handle<Image>,

    /// The current image pixel size.
    pub size: u32,
}

/// This system prepares the camera for rendering block previews in the Block
/// Editor UI.
pub fn prepare_camera(
    block_finger: BlockFinder,
    mut images: ResMut<Assets<Image>>,
    mut egui_textures: ResMut<EguiUserTextures>,
    mut commands: Commands,
) {
    let size = Extent3d {
        width: BLOCK_PREVIEW_SIZE,
        height: BLOCK_PREVIEW_SIZE,
        ..default()
    };

    let background_color = Color::srgb(0.5, 0.5, 0.5);

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);

    let image_handle = images.add(image);
    egui_textures.add_image(image_handle.clone());
    commands.insert_resource(BlockPreviewWidget {
        handle: image_handle.clone(),
        size: BLOCK_PREVIEW_SIZE,
    });

    // light source
    commands.spawn((
        BlockPreviewElement,
        RenderLayers::layer(2),
        DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                67.5f32.to_radians(),
                22.5f32.to_radians(),
                0f32.to_radians(),
            )),
            ..default()
        },
    ));

    // camera
    commands.spawn((
        BlockPreviewElement,
        RenderLayers::layer(2),
        Camera3dBundle {
            camera: Camera {
                order: 1,
                clear_color: background_color.into(),
                target: RenderTarget::Image(image_handle),
                ..default()
            },
            projection: OrthographicProjection {
                near: -10.0,
                far: 10.0,
                scaling_mode: ScalingMode::Fixed {
                    width: 3f32.sqrt() * BLOCK_PREVIEW_SCALE,
                    height: 3f32.sqrt() * BLOCK_PREVIEW_SCALE,
                },
                viewport_origin: Vec2::new(0.5, 0.5),
                ..default()
            }
            .into(),
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::YXZ,
                45f32.to_radians(),
                -45f32.to_radians(),
                0.0,
            )),
            ..default()
        },
    ));

    // block
    commands.spawn((
        BlockPreviewElement,
        RenderLayers::layer(2),
        RenderedBlock {
            block: block_finger.find("grass").unwrap(),
        },
        PbrBundle {
            transform: Transform::from_translation(Vec3::splat(-0.5)),
            ..default()
        },
    ));
}

/// This system cleans up the camera used to render block previews in the Block
/// Editor UI.
pub fn cleanup_camera(
    mut egui_textures: ResMut<EguiUserTextures>,
    preview_handle: Res<BlockPreviewWidget>,
    elements: Query<Entity, With<BlockPreviewElement>>,
    mut commands: Commands,
) {
    for entity in elements.iter() {
        commands.entity(entity).despawn_recursive();
    }

    egui_textures.remove_image(&preview_handle.handle);
    commands.remove_resource::<BlockPreviewWidget>();
}

/// This system is called when the Block Editor UI screen is opened to enable
/// the camera used to render block previews.
pub fn enable_camera(mut camera: Query<&mut Camera, With<BlockPreviewElement>>) {
    let Ok(mut cam) = camera.get_single_mut() else {
        return;
    };
    cam.is_active = true;
}

/// This system is called when the Block Editor UI screen is closed to disable
/// the camera used to render block previews.
pub fn disable_camera(mut camera: Query<&mut Camera, With<BlockPreviewElement>>) {
    let Ok(mut cam) = camera.get_single_mut() else {
        return;
    };
    cam.is_active = false;
}
