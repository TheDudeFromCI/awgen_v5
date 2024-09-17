//! This module implements the camera plugin.

use bevy::prelude::*;

/// The plugin responsible for managing the camera.
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(PreStartup, setup_camera);
    }
}

/// Spawns a camera.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
