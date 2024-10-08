//! This module implements support for rendering a 3D models in the UI. This is
//! useful for things such as displaying a 3D block model in the Hotbar, for
//! example.

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::render::view::RenderLayers;
use bevy::window::WindowRef;

use crate::camera::CAMERA_CLIP_DIST;

/// This is a marker component used to indicate that the camera is used to
/// render 3D models in UI elements.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct UI3DCamera;

/// This system creates the 3d icon camera.
pub fn setup_icon3d_camera(mut commands: Commands) {
    commands.spawn((
        RenderLayers::layer(1),
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

    commands.spawn((
        UI3DCamera,
        RenderLayers::layer(1),
        Camera3dBundle {
            camera: Camera {
                order: 1,
                clear_color: Color::NONE.into(),
                target: RenderTarget::Window(WindowRef::Primary),
                ..default()
            },
            projection: OrthographicProjection {
                near: -CAMERA_CLIP_DIST,
                far: CAMERA_CLIP_DIST,
                scaling_mode: ScalingMode::WindowSize(1.0),
                viewport_origin: Vec2::new(0.0, 1.0),
                ..default()
            }
            .into(),
            transform: Transform::from_rotation(Quat::from_rotation_x(PI)),
            ..default()
        },
    ));
}
