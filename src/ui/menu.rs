//! This module implements that main menu game states and routing logic.

use bevy::prelude::*;

/// The plugin responsible for managing the main menu UI.
///
/// Adding this plugin will automatically all child plugins.
pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_state::<MainMenuState>()
            .add_plugins(super::splash::SplashPlugin);
    }
}

/// The main menu state machine.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum MainMenuState {
    /// The splash screen state. This is the default menu state used on startup.
    #[default]
    Splash,

    /// The project editor menu.
    Editor,
}
