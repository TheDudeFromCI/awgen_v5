//! This module contains all the events that can be sent to the AwgenScript
//! engine.

use serde::{Deserialize, Serialize};

use crate::blocks::tileset::TilesetDefinition;

/// An enum that represents all possible events that can be sent to the
/// AwgenScript engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum LogicEvent {
    /// An event that is triggered once when the engine is first started.
    /// Contains the project settings information.
    EngineStarted {
        /// The name of the project.
        project_name: String,

        /// The version of the project.
        project_version: String,

        /// A list of all the tilesets in the project.
        tilesets: Vec<TilesetDefinition>,
    },
}

impl LogicEvent {
    /// Converts the input into a JSON string.
    pub fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
