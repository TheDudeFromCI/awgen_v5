//! This module implements the camera plugin.

use std::f32::consts::{PI, TAU};

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::ui::menu::MainMenuState;

/// The duration of the camera lerp in seconds.
pub const CAMERA_SMOOTH_DUR: f32 = 0.05;

/// The distance from the camera to the clipping plane.
pub const CAMERA_HALF_DIST: f32 = 500.0;

/// The minimum zoom level of the camera.
pub const MIN_ZOOM: f32 = 4.0;

/// The maximum zoom level of the camera.
pub const MAX_ZOOM: f32 = 256.0;

/// The plugin responsible for managing the camera.
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(PreStartup, setup_camera).add_systems(
            Update,
            (
                smooth_camera_lerp,
                camera_controls.run_if(MainMenuState::is_in_game),
            ),
        );
    }
}

/// This is a marker component that indicates an entity is the main camera.
#[derive(Debug, Default, Component)]
pub struct MainCamera;

/// This component stores the target position and angle of the camera. The
/// camera will lerp to this position and angle over time.
#[derive(Debug, Component)]
pub struct CameraTarget {
    /// The target position of the camera. This value should the the focal point
    /// of the camera.
    pub pos: Vec3,

    /// The target yaw of the camera.
    pub yaw: f32,

    /// The zoom level of the camera.
    pub zoom: f32,
}

impl Default for CameraTarget {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            yaw: PI / 4.0,
            zoom: 16.0,
        }
    }
}

/// Spawns a camera.
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        CameraTarget::default(),
        Camera3dBundle {
            projection: OrthographicProjection {
                near: -CAMERA_HALF_DIST,
                far: CAMERA_HALF_DIST,
                scaling_mode: ScalingMode::FixedVertical(1.0),
                scale: 16.0,
                ..default()
            }
            .into(),
            ..default()
        },
    ));
}

/// This system lerps the camera to its target position and angle.
fn smooth_camera_lerp(
    time: Res<Time>,
    mut cameras: Query<(&mut Transform, &mut Projection, &CameraTarget)>,
) {
    let delta = (time.delta_seconds() / CAMERA_SMOOTH_DUR).clamp(0.0, 1.0);

    for (mut transform, mut projection, target) in cameras.iter_mut() {
        transform.translation = transform.translation.lerp(target.pos, delta);

        let rot = Quat::from_euler(EulerRot::YXZ, target.yaw, -PI / 4.0, 0.0);
        transform.rotation = transform.rotation.slerp(rot, delta);

        if let Projection::Orthographic(proj) = &mut *projection {
            proj.scale = proj.scale * (target.zoom / proj.scale).powf(delta);
        };
    }
}

/// This system listens for keyboard inputs and moves the camera accordingly.
fn camera_controls(
    key_input: Res<ButtonInput<KeyCode>>,
    mut wheel: EventReader<MouseWheel>,
    mut cameras: Query<&mut CameraTarget, With<MainCamera>>,
) {
    let Ok(mut camera) = cameras.get_single_mut() else {
        return;
    };

    if key_input.just_pressed(KeyCode::KeyQ) {
        camera.yaw = (camera.yaw - PI / 4.0) % TAU;
    }

    if key_input.just_pressed(KeyCode::KeyE) {
        camera.yaw = (camera.yaw + PI / 4.0) % TAU;
    }

    for ev in wheel.read() {
        camera.zoom = (camera.zoom / 1.25f32.powf(ev.y)).clamp(MIN_ZOOM, MAX_ZOOM);
    }
}
