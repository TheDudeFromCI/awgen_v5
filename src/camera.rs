//! This module implements the camera plugin.

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::ui::menu::MainMenuState;

/// The distance from the camera to the clipping plane. (In both directions)
pub const CAMERA_CLIP_DIST: f32 = 500.0;

/// The base zoom level of the camera.
pub const BASE_ZOOM: f32 = 16.0;

/// The minimum zoom level of the camera.
pub const MIN_ZOOM: f32 = 4.0 / BASE_ZOOM;

/// The maximum zoom level of the camera.
pub const MAX_ZOOM: f32 = 256.0 / BASE_ZOOM;

/// The minimum pitch of the camera. (In radians)
pub const MIN_PITCH: f32 = -80.0;

/// The maximum pitch of the camera. (In radians)
pub const MAX_PITCH: f32 = -22.5;

/// The plugin responsible for managing the camera.
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(PreStartup, setup_camera)
            .add_systems(
                Update,
                (mouse_pan, mouse_rotate, mouse_zoom).in_set(CameraSystemSets::Controls),
            )
            .add_systems(
                Update,
                smooth_camera_lerp.in_set(CameraSystemSets::Smoothing),
            )
            .configure_sets(
                Update,
                CameraSystemSets::Controls
                    .before_ignore_deferred(CameraSystemSets::Smoothing)
                    .run_if(MainMenuState::is_in_game),
            );
    }
}

/// The system sets used within the camera plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum CameraSystemSets {
    /// Systems that handle user input controlling the camera.
    Controls,

    /// Systems that smoothly move the target from it's current position to the
    /// target position.
    Smoothing,
}

/// This is a marker component that indicates an entity is the main camera.
#[derive(Debug, Default, Component)]
pub struct MainCamera;

/// An entity with this component is used to indicate the target position of the
/// main camera. The main camera will smoothly lerp to this position of this
/// entity over the course of `duration` seconds.
///
/// This component should only be attached to a single entity.
///
/// The scale of the target entity is used to control the zoom level of the
/// camera. Only the x component of the scale is used.
#[derive(Debug, Component)]
pub struct CameraTarget {
    /// The number of seconds it takes for the camera to lerp to the target.
    pub duration: f32,

    /// The rotation of the camera in euler angles. (In radians)
    ///
    /// This value is used to calculate the quaternion rotation of this entity.
    /// Rotating the entity directly will not have any effect.
    pub rotation: Vec3,
}

impl Default for CameraTarget {
    fn default() -> Self {
        Self {
            duration: 0.05,
            rotation: Vec3::new(45.0, -45.0, 0.0),
        }
    }
}

impl CameraTarget {
    /// Calculates the quaternion rotation of the camera target.
    pub fn rotation(&self) -> Quat {
        Quat::from_euler(
            EulerRot::YXZ,
            self.rotation.x.to_radians(),
            self.rotation.y.to_radians(),
            self.rotation.z.to_radians(),
        )
    }

    /// Returns the up vector of the camera target.
    pub fn up(&self) -> Vec3 {
        self.rotation() * Vec3::Y
    }

    /// Returns the right vector of the camera target.
    pub fn right(&self) -> Vec3 {
        self.rotation() * Vec3::X
    }
}

/// The control component for the camera. This component should be added to the
/// [`CameraTarget`] entity. This component contains the settings that are used
/// to defined the user input controls for the camera.
#[derive(Debug, Component)]
pub struct CameraControls {
    /// The drag-pan sensitivity of the camera.
    pub pan_sensitivity: f32,

    /// The drag-rotate sensitivity of the camera.
    pub rotate_sensitivity: f32,

    /// The zoom sensitivity of the camera.
    pub zoom_sensitivity: f32,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            pan_sensitivity: 1.0,
            rotate_sensitivity: 0.25,
            zoom_sensitivity: 1.0,
        }
    }
}

/// Spawns a camera.
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera3dBundle {
            projection: OrthographicProjection {
                near: -CAMERA_CLIP_DIST,
                far: CAMERA_CLIP_DIST,
                scaling_mode: ScalingMode::FixedVertical(BASE_ZOOM),
                scale: 1.0,
                ..default()
            }
            .into(),
            ..default()
        },
    ));

    commands.spawn((
        CameraTarget::default(),
        CameraControls::default(),
        SpatialBundle::default(),
    ));
}

/// This system lerps the camera to its target position and angle.
fn smooth_camera_lerp(
    time: Res<Time>,
    cam_target: Query<(&Transform, &CameraTarget), Without<MainCamera>>,
    mut main_cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
    let (mut cam_transform, mut projection) = main_cam.single_mut();
    let (target_pos, target_props) = cam_target.single();

    let delta = (time.delta_seconds() / target_props.duration).clamp(0.0, 1.0);

    cam_transform.translation = cam_transform
        .translation
        .lerp(target_pos.translation, delta);

    cam_transform.rotation = cam_transform.rotation.slerp(target_props.rotation(), delta);

    if let Projection::Orthographic(proj) = &mut *projection {
        proj.scale = proj.scale * (target_pos.scale.x / proj.scale).powf(delta);
    };
}

/// This system listens for mouse movement inputs and pans the camera
/// accordingly. The camera panning is only active when the right mouse button
/// is pressed.
fn mouse_pan(
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    main_cam: Query<(&Camera, &Projection), With<MainCamera>>,
    mut cam_target: Query<(&mut Transform, &CameraTarget, &CameraControls)>,
) {
    if !mouse_button.pressed(MouseButton::Right) {
        return;
    }

    let (camera, projection) = main_cam.single();
    let (mut target_pos, target_props, controls) = cam_target.single_mut();

    let mut delta = mouse_motion.read().map(|e| e.delta).sum::<Vec2>();
    let vp_size = camera.logical_viewport_size().unwrap_or(Vec2::ONE);

    match projection {
        Projection::Orthographic(ortho) => {
            delta *= ortho.area.size() / vp_size;
            delta *= controls.pan_sensitivity;
        }
        _ => unreachable!("Camera should be orthographic"),
    }

    target_pos.translation += target_props.up() * delta.y;
    target_pos.translation += target_props.right() * -delta.x;
}

/// This system listens for mouse movement inputs and rotates the camera
/// accordingly. The camera rotation is only active when the middle mouse button
/// is pressed.
fn mouse_rotate(
    mut mouse_motion: EventReader<MouseMotion>,
    button: Res<ButtonInput<MouseButton>>,
    mut cam_target: Query<(&mut CameraTarget, &CameraControls)>,
) {
    if !button.pressed(MouseButton::Middle) {
        return;
    }

    let (mut target, controls) = cam_target.single_mut();

    let mut delta = mouse_motion.read().map(|e| e.delta).sum::<Vec2>();
    delta *= controls.rotate_sensitivity;

    target.rotation.x = (target.rotation.x - delta.x) % 360.0;
    target.rotation.y = (target.rotation.y - delta.y).clamp(MIN_PITCH, MAX_PITCH);
}

/// This system listens for mouse wheel inputs and zooms the camera accordingly.
fn mouse_zoom(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut cam_target: Query<(&mut Transform, &CameraControls)>,
) {
    let (mut target_pos, target_props) = cam_target.single_mut();
    let mut delta = mouse_wheel.read().map(|e| e.y).sum::<f32>();
    delta *= target_props.zoom_sensitivity;

    target_pos.scale.x = (target_pos.scale.x * 1.25f32.powf(-delta)).clamp(MIN_ZOOM, MAX_ZOOM);
}
