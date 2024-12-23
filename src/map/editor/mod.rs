//! This module implements the map editor.

use bevy::prelude::*;

use crate::gamestate::GameState;
use crate::gizmos::GizmoSystemSets;
use crate::ui::EditorWindowState;

pub mod placement;
pub mod startup;

/// The map editor plugin. This plugin allows for the user to directly edit the
/// world.
pub struct MapEditorPlugin;
impl Plugin for MapEditorPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(OnEnter(GameState::Editor), startup::prepare_map_editor)
            .add_systems(
                Update,
                (
                    placement::place_block.in_set(MapEditorSystemSets::PlaceBlock),
                    placement::remove_block.in_set(MapEditorSystemSets::RemoveBlock),
                ),
            )
            .configure_sets(
                Update,
                (
                    MapEditorSystemSets::RemoveBlock
                        .after_ignore_deferred(GizmoSystemSets::UpdateCursor)
                        .run_if(in_state(EditorWindowState::MapEditor)),
                    MapEditorSystemSets::PlaceBlock
                        .after_ignore_deferred(GizmoSystemSets::UpdateCursor)
                        .after_ignore_deferred(MapEditorSystemSets::RemoveBlock)
                        .run_if(in_state(EditorWindowState::MapEditor)),
                ),
            );
    }
}

/// The system sets for the map editor plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum MapEditorSystemSets {
    /// The system set for placing blocks.
    PlaceBlock,

    /// The system set for removing blocks.
    RemoveBlock,
}
