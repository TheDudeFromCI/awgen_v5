//! This module implements the functionality for rendering 3D icons in the UI.

use bevy::prelude::*;

pub mod renderer;

/// This plugin adds the 3D icon rendering systems and components to the app.
pub struct Icon3DPlugin;
impl Plugin for Icon3DPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(Startup, renderer::setup_icon3d_camera);
    }
}
