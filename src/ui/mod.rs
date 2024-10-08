//! This module contains various UI related components and systems.

pub mod gui3d;
pub mod hotbar;
pub mod menu;
pub mod splash;

use bevy::prelude::*;

/// The plugin that adds the UI systems and components to the app.
pub struct AwgenUIPlugin;
impl Plugin for AwgenUIPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_plugins((
            gui3d::Icon3DPlugin,
            menu::MainMenuPlugin,
            hotbar::UiHotbarPlugin,
        ));
    }
}
