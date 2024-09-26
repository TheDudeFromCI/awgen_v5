//! This module implements gizmos that are used to let the user interact with
//! the world and entities within it.

use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::ui::menu::MainMenuState;

pub mod cursor;
pub mod face;

/// This plugin implements Gizmos functionality and management systems.
pub struct GizmosPlugin;
impl Plugin for GizmosPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_resource::<cursor::CursorRaycast>()
            .add_systems(OnEnter(MainMenuState::Editor), face::build_block_face_gizmo)
            .add_systems(
                Update,
                (
                    cursor::update_cursor_block.in_set(GizmoSystemSets::UpdateCursor),
                    face::update_block_face_gizmo.in_set(GizmoSystemSets::BlockFaceGizmo),
                    face::animate_block_face_gizmo.in_set(GizmoSystemSets::BlockFaceGizmo),
                ),
            )
            .configure_sets(
                Update,
                GizmoSystemSets::BlockFaceGizmo
                    .after_ignore_deferred(GizmoSystemSets::UpdateCursor),
            );

        embedded_asset!(app_, "block_face.glb");
    }
}

/// The system sets for the gizmos plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum GizmoSystemSets {
    /// The system set for updating the raycast cursor resource.
    UpdateCursor,

    /// The system set for updating block face gizmos.
    BlockFaceGizmo,
}
