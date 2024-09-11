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
    commands.spawn(Camera3dBundle::default());
}
