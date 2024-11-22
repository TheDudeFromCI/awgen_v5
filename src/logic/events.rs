//! This module contains the messages that can be sent to the AwgenScript
//! engine.

use serde::{Deserialize, Serialize};

/// The logic input enum represents all possible inputs that can be sent to the
/// logic system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LogicEvent {
    /// The AwgenScript engine has started. This input is sent once directly
    /// after the engine has been initialized.
    #[serde(rename = "engine_started")]
    EngineStarted,

    /// A packet containing the current project settings.
    #[serde(rename = "project_settings")]
    ProjectSettings {
        /// The name of the project.
        name: String,

        /// The version of the project.
        version: String,
    },
}

impl LogicEvent {
    /// Converts the input into a JSON string.
    pub fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
