//! The game state module.

use bevy::prelude::*;

/// The current game state.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    /// The initial state. This state is active only for a single frame while
    /// the application window is loading. This is the default menu state used
    /// on startup.
    #[default]
    Init,

    /// The splash screen state.
    Splash,

    /// The project editor state.
    #[cfg(feature = "editor")]
    Editor,

    /// The project runtime state.
    Runtime,
}

impl GameState {
    /// A runtime condition that returns true if the game is in a playable
    /// state. (Editor or Player)
    pub fn is_playing(state: Res<State<GameState>>) -> bool {
        matches!(**state, GameState::Editor | GameState::Runtime)
    }
}

/// This system runs on startup to transition to the splash screen.
pub fn to_splash_screen(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Splash);
}
