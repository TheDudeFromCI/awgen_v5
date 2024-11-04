//! This module implements the block preview widget for the Block Editor UI
//! screen.

use bevy::math::bounding::{Aabb3d, RayCast3d};
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
use crate::math::FaceDirection;

/// The asset path to the block face rotation gizmo model.
pub const GIZMO_FACESEL_MODEL: &str = "embedded://awgen/ui/block_editor/block_face_rotation.glb";

/// The default image size for the block preview widget in the Block Editor UI.
pub const BLOCK_PREVIEW_SIZE: u32 = 300;

/// The scale factor used to render block previews in the Block Editor UI. A
/// scale factor of 1.0 indicates that the block preview will exactly large
/// enough to fit a block at a isometric angle. A value greater than 1.0 will
/// add a percentage of padding around the block preview.
pub const BLOCK_PREVIEW_SCALE: f32 = 1.5;

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

/// This is a marker component used to indicate that the entity is the face
/// selection gizmo model that is used to select block faces in the Block Editor
/// UI.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct BlockSelectedFaceModel;

/// This resource contains information about the block preview widget used in
/// the Block Editor UI screen. This resource may not exist when the game is not
/// in the Editor game state.
#[derive(Debug, Resource)]
pub struct BlockPreviewWidget {
    /// The image handle of the target render texture.
    handle: Handle<Image>,

    /// The current image pixel size.
    size: u32,

    /// The local rotation euler angles of the camera. Measured in radians.
    rotation: Vec2,

    /// The currently active block entity.
    active_block: Entity,

    /// The currently selected face of the block, if any.
    selected_face: Option<FaceDirection>,

    /// The currently hovered face of the block, if any.
    hover_face: Option<FaceDirection>,

    /// The local mouse position within the block preview widget.
    local_mouse_pos: Vec2,
}

impl BlockPreviewWidget {
    /// Rotates the camera by the given pitch and yaw angles.
    pub fn rotate(&mut self, pitch: f32, yaw: f32) {
        self.rotation.x += pitch.to_radians() * DRAG_SENSITIVITY;
        self.rotation.y += yaw.to_radians() * DRAG_SENSITIVITY;
        trace!("Block preview camera rotation: {}", self.rotation);
    }

    /// Returns the current rotation of the camera as a quaternion.
    pub fn get_rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.rotation.x, self.rotation.y, 0.0)
    }

    /// Returns the current size of the image in pixels.
    pub fn get_size(&self) -> u32 {
        self.size
    }

    /// Returns the current image handle.
    pub fn get_handle(&self) -> Handle<Image> {
        self.handle.clone()
    }

    /// Sets the active block entity, replacing the current active block.
    pub fn set_active_block(&mut self, block: Entity) {
        self.active_block = block;
    }

    /// Sets the currently selected face of the block, or `None` to deselect.
    pub fn set_selected_face(&mut self, face: Option<FaceDirection>) {
        self.selected_face = face;
    }

    /// Returns the currently selected face of the block, if any.
    pub fn get_hovered_face(&self) -> Option<FaceDirection> {
        self.hover_face
    }

    /// Sets the local mouse position within the block preview widget.
    pub fn set_mouse_pos(&mut self, pos: Vec2) {
        self.local_mouse_pos = pos;
        trace!("Block preview widget mouse position: {pos}");
    }
}

/// This system prepares the camera for rendering block previews in the Block
/// Editor UI.
pub fn prepare_camera(
    block_finger: BlockFinder,
    asset_server: Res<AssetServer>,
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
    let air_id = block_finger.find_air();

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
        selected_face: None,
        hover_face: None,
        local_mouse_pos: Vec2::ZERO,
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

    // face selection gizmo
    commands.spawn((
        BlockPreviewElement,
        BlockSelectedFaceModel,
        RenderLayers::layer(2),
        SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(GIZMO_FACESEL_MODEL)),
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

/// This system is called when the active block widget is changed in the Block
/// Editor UI screen. This will update the block preview widget with the new
/// model and visuals.
#[allow(clippy::type_complexity)]
pub fn update_preview(
    preview_widget: Res<BlockPreviewWidget>,
    mut model: Query<&mut RenderedBlock, With<BlockPreviewModel>>,
    mut camera: Query<
        &mut Transform,
        (
            With<Camera>,
            With<BlockPreviewElement>,
            Without<BlockSelectedFaceModel>,
        ),
    >,
    mut face_gizmo: Query<
        (&mut Transform, &mut Visibility),
        (With<BlockSelectedFaceModel>, Without<Camera>),
    >,
) {
    let mut rendered_block = model.single_mut();
    if rendered_block.block != preview_widget.active_block {
        rendered_block.block = preview_widget.active_block;
        trace!(
            "Updating block preview widget with new block model {:?}",
            preview_widget.active_block
        );
    }

    camera.single_mut().rotation = preview_widget.get_rotation();

    let (mut face_transform, mut face_visibility) = face_gizmo.single_mut();
    if let Some(face) = preview_widget.selected_face {
        *face_visibility = Visibility::Inherited;
        face_transform.rotation = face.rotation_quat();
    } else {
        *face_visibility = Visibility::Hidden;
    }
}

/// This system listens for when the scene bundle for the face selection gizmo
/// model is loaded and updates the render layer of the gizmo model.
pub fn update_gizmo_render_layer(
    mut asset_load_evs: EventReader<AssetEvent<Scene>>,
    face_gizmo: Query<(Entity, &Handle<Scene>), With<BlockSelectedFaceModel>>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    for ev in asset_load_evs.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            let Ok((root_entity, handle)) = face_gizmo.get_single() else {
                return;
            };

            if *id != handle.id() {
                return;
            }

            for entity in children.iter_descendants(root_entity) {
                commands.entity(entity).insert(RenderLayers::layer(2));
            }
        }
    }
}

/// This system updates the currently hovered face of the block preview widget.
pub fn update_face_hover(
    mut widget: ResMut<BlockPreviewWidget>,
    camera: Query<(&Camera, &GlobalTransform), With<BlockPreviewElement>>,
) {
    let (cam, cam_transform) = camera.single();
    let Some(pos) = cam.viewport_to_world(cam_transform, widget.local_mouse_pos) else {
        widget.hover_face = None;
        return;
    };

    let raycast = RayCast3d::new(pos.origin, pos.direction, 20.0);
    let aabb = Aabb3d::new(Vec3::ZERO, Vec3::splat(0.5));

    let new_face = {
        if let Some(dist) = raycast.aabb_intersection_at(&aabb) {
            let hit = pos.origin + pos.direction * dist;
            FaceDirection::from_normal(hit)
        } else {
            None
        }
    };

    if widget.hover_face == new_face {
        return;
    }

    trace!("Block preview widget hover face: {:?}", new_face);
    widget.hover_face = new_face;
}
