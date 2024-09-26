//! This module implements gizmos that are used to let the user interact with
//! the world and entities within it.

use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::ui::menu::MainMenuState;

pub mod face;

/// This plugin implements Gizmos functionality and management systems.
pub struct GizmosPlugin;
impl Plugin for GizmosPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(OnEnter(MainMenuState::Editor), face::build_block_face_gizmo)
            .add_systems(
                Update,
                (
                    face::update_block_face_gizmo,
                    face::animate_block_face_gizmo,
                ),
            );

        embedded_asset!(app_, "block_face.glb");
    }
}
