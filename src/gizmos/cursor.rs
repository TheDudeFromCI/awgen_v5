//! This module implement a resource for continuous voxel raycasting the cursor,
//! to prevent having to recalculate the raycast multiple times per frame.

use bevy::math::bounding::RayCast3d;
use bevy::prelude::*;

use crate::camera::{MainCamera, CAMERA_CLIP_DIST};
use crate::utilities::raycast::{VoxelRaycast, VoxelRaycastHit};

/// The distance to raycast from the cursor.
const RAYCAST_DISTANCE: f32 = CAMERA_CLIP_DIST * 2.0;

/// The resource that stores the current cursor raycast information.
///
/// This resource is updated every frame and stores the object currently under
/// the mouse cursor.
#[derive(Debug, Default, Resource)]
pub struct CursorRaycast {
    /// The current block that the cursor is hovering over, if any.
    pub block: Option<VoxelRaycastHit>,
}

/// This system runs every frame to update the cursor block based on the current
/// cursor position.
pub fn update_cursor_block(
    mut cursor: ResMut<CursorRaycast>,
    raycast: VoxelRaycast,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    window: Query<&Window>,
) {
    let Ok((camera, cam_transform)) = camera.get_single() else {
        return;
    };

    let Ok(window) = window.get_single() else {
        return;
    };

    let Some(mouse_pos) = window.cursor_position() else {
        return;
    };

    let Some(ray) = camera.viewport_to_world(cam_transform, mouse_pos) else {
        return;
    };

    cursor.block = raycast.raycast(RayCast3d::new(ray.origin, ray.direction, RAYCAST_DISTANCE));
}
