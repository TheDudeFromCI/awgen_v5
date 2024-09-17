//! This module implements that main menu game states and routing logic.

use bevy::prelude::*;

/// The plugin responsible for managing the main menu UI.
///
/// Adding this plugin will automatically all child plugins.
pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_state::<MainMenuState>()
            .add_plugins(super::splash::SplashPlugin)
            .add_systems(Startup, to_splash_screen);
    }
}

/// The main menu state machine.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum MainMenuState {
    /// The initial state. This state is active only for a single frame while
    /// the application window is loading. This is the default menu state used
    /// on startup.
    #[default]
    Init,

    /// The splash screen state.
    Splash,

    /// The project editor menu.
    Editor,
}

/// This system runs on startup to transition to the splash screen.
fn to_splash_screen(mut state: ResMut<NextState<MainMenuState>>) {
    state.set(MainMenuState::Splash);
}
