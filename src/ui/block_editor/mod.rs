//! This module implements the Block Editor UI screen within the editor mode.

use bevy::prelude::*;

use super::{EditorWindowState, GameState};

pub mod ui;

/// The plugin that adds the Block Editor UI systems and components to the app.
pub struct BlockEditorUiPlugin;
impl Plugin for BlockEditorUiPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(
            Update,
            (
                ui::build
                    .after_ignore_deferred(ui::open)
                    .after_ignore_deferred(ui::close)
                    .run_if(in_state(GameState::Editor))
                    .run_if(in_state(EditorWindowState::BlockEditor)),
                ui::open
                    .run_if(in_state(GameState::Editor))
                    .run_if(not(in_state(EditorWindowState::BlockEditor))),
                ui::close
                    .run_if(in_state(GameState::Editor))
                    .run_if(in_state(EditorWindowState::BlockEditor)),
            ),
        );
    }
}
