//! This module contains various UI related components and systems.

#[cfg(feature = "editor")]
pub mod block_editor;
pub mod gui3d;
pub mod hotbar;
pub mod splash;

use bevy::prelude::*;

/// The plugin that adds the UI systems and components to the app.
pub struct AwgenUIPlugin;
impl Plugin for AwgenUIPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_state::<EditorWindowState>().add_plugins((
            #[cfg(feature = "editor")]
            block_editor::BlockEditorUiPlugin,
            gui3d::Icon3DPlugin,
            hotbar::UiHotbarPlugin,
            splash::SplashPlugin,
        ));
    }
}

/// This system runs on startup to transition to the splash screen.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum EditorWindowState {
    /// The map editor window.
    #[default]
    MapEditor,

    /// The block editor window.
    BlockEditor,
}
