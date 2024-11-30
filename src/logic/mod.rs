//! This module implements the scripting engine and logic for the game. All
//! logic is received through the JavaScript runtime, which is then translated
//! into commands and executed on the game state.

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use resources::AwgenScriptChannels;

use crate::gamestate::GameState;

pub mod api;
pub mod channels;
pub mod commands;
pub mod events;
pub mod queries;
pub mod queue;
pub mod resources;
pub mod systems;

/// The logic plugin is responsible for handling all game logic. This includes
/// the scripting engine, which is used to run the game's logic.
pub struct LogicPlugin;
impl Plugin for LogicPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_resource::<LogicPluginSettings>()
            .init_resource::<AwgenScriptChannels>()
            .add_systems(OnEnter(GameState::Runtime), systems::begin_runtime_loop)
            .add_systems(OnExit(GameState::Runtime), systems::close_engine_loop)
            .add_systems(
                Update,
                systems::handle_logic_outputs.run_if(resource_exists::<AwgenScriptChannels>),
            );

        #[cfg(feature = "editor")]
        {
            app_.add_systems(OnEnter(GameState::Editor), systems::begin_editor_loop)
                .add_systems(OnExit(GameState::Editor), systems::close_engine_loop);
        }
    }
}

/// The logic plugin settings resource.
#[derive(Debug, Resource)]
pub struct LogicPluginSettings {
    /// The path to the editor script source folder.
    #[cfg(feature = "editor")]
    pub editor_script_path: PathBuf,

    /// The path to the runtime script source folder.
    pub runtime_script_path: PathBuf,
}

impl Default for LogicPluginSettings {
    fn default() -> Self {
        Self {
            #[cfg(feature = "editor")]
            editor_script_path: Path::new("./assets/editor_scripts").to_path_buf(),
            runtime_script_path: Path::new("./scripts").to_path_buf(),
        }
    }
}
