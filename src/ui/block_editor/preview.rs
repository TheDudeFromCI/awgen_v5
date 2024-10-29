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

use crate::blocks::params::BlockFinder;
use crate::blocks::{AIR_BLOCK_NAME, RenderedBlock};

/// The default image size for the block preview widget in the Block Editor UI.
pub const BLOCK_PREVIEW_SIZE: u32 = 300;

/// The scale factor used to render block previews in the Block Editor UI. A
/// scale factor of 1.0 indicates that the block preview will exactly large
/// enough to fit a block at a isometric angle. A value greater than 1.0 will
/// add a percentage of padding around the block preview.
pub const BLOCK_PREVIEW_SCALE: f32 = 1.1;

/// The sensitivity of the drag input used to rotate the block preview camera.
pub const DRAG_SENSITIVITY: f32 = 0.5;

/// This is a marker component used to indicate that the entity is used to
/// render block previews in the Block Editor UI.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct BlockPreviewElement;

/// This is a marker component used to indicate that the entity is the block
/// model that is previewed in the Block Editor UI.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct BlockPreviewModel;

/// This resource contains information about the block preview widget used in
/// the Block Editor UI screen. This resource may not exist when the game is not
/// in the Editor game state.
#[derive(Debug, Resource)]
pub struct BlockPreviewWidget {
    /// The image handle of the target render texture.
    pub handle: Handle<Image>,

    /// The current image pixel size.
    pub size: u32,

    /// The local rotation euler angles of the camera. Measured in radians.
    pub rotation: Vec2,

    /// The currently active block entity.
    pub active_block: Entity,
}

impl BlockPreviewWidget {
    /// Rotates the camera by the given pitch and yaw angles.
    pub fn rotate(&mut self, pitch: f32, yaw: f32) {
        trace!("Block preview camera rotation: {pitch}, {yaw}");
        self.rotation.x += pitch.to_radians() * DRAG_SENSITIVITY;
        self.rotation.y += yaw.to_radians() * DRAG_SENSITIVITY;
    }

    /// Returns the current rotation of the camera as a quaternion.
    pub fn get_rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.rotation.x, self.rotation.y, 0.0)
    }
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
    let air_id = block_finger.find(AIR_BLOCK_NAME).unwrap();

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

    let widget = BlockPreviewWidget {
        handle: image_handle.clone(),
        size: BLOCK_PREVIEW_SIZE,
        rotation: Vec2::new(45f32.to_radians(), -45f32.to_radians()),
        active_block: air_id,
    };

    // camera
    commands
        .spawn((
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
                transform: Transform::from_rotation(widget.get_rotation()),
                ..default()
            },
        ))
        .with_children(|parent| {
            // light source
            parent.spawn((
                BlockPreviewElement,
                RenderLayers::layer(2),
                DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        illuminance: light_consts::lux::FULL_DAYLIGHT,
                        ..default()
                    },
                    transform: Transform::from_rotation(Quat::from_euler(
                        EulerRot::XYZ,
                        -30f32.to_radians(),
                        30f32.to_radians(),
                        0f32.to_radians(),
                    )),
                    ..default()
                },
            ));
        });

    // block
    commands.spawn((
        BlockPreviewElement,
        BlockPreviewModel,
        RenderLayers::layer(2),
        RenderedBlock { block: air_id },
        PbrBundle {
            transform: Transform::from_translation(Vec3::splat(-0.5)),
            ..default()
        },
    ));

    commands.insert_resource(widget);
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

/// This system is called when the active block is changed in the Block Editor
/// UI screen. This will update the block preview widget with the new model.
pub fn update_selected_block(
    preview_widget: Res<BlockPreviewWidget>,
    mut model: Query<&mut RenderedBlock, With<BlockPreviewModel>>,
) {
    let mut rendered_block = model.single_mut();
    rendered_block.block = preview_widget.active_block;
}
