//! This module contains various UI related components and systems.

pub mod block_editor;
pub mod gui3d;
pub mod hotbar;
pub mod splash;

use bevy::prelude::*;

/// The plugin that adds the UI systems and components to the app.
pub struct AwgenUIPlugin;
impl Plugin for AwgenUIPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_state::<GameState>()
            .init_state::<EditorWindowState>()
            .add_plugins((
                block_editor::BlockEditorUiPlugin,
                gui3d::Icon3DPlugin,
                hotbar::UiHotbarPlugin,
                splash::SplashPlugin,
            ))
            .add_systems(Startup, to_splash_screen);
    }
}

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
    Editor,

    /// The in-game player state.
    Player,
}

impl GameState {
    /// A runtime condition that returns true if the game is in a playable
    /// state. (Editor or Player)
    pub fn is_playing(state: Res<State<GameState>>) -> bool {
        matches!(**state, GameState::Editor | GameState::Player)
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

/// This system runs on startup to transition to the splash screen.
pub fn to_splash_screen(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Splash);
}
