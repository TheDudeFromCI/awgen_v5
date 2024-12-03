//! This module contains the commands that can be received from the AwgenScript
//! engine.

use bevy::log::error;
use boa_engine::{Context, JsValue};
use serde::{Deserialize, Serialize};

/// An enum that represents all possible commands that can be received from the
/// AwgenScript engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "camelCase")]
pub enum LogicCommands {
    /// A command that is used to update the project name.
    SetProjectName {
        /// The new name of the project.
        name: String,
    },

    /// A command that is used to update the project version.
    SetProjectVersion {
        /// The new version of the project.
        version: String,
    },
}

impl LogicCommands {
    /// Converts the given [`JsValue`] into a [`LogicCommands`] instance, if
    /// possible. Returns `None` if the conversion fails.
    pub fn from_js_value(value: &JsValue, context: &mut Context) -> Option<Self> {
        let json = value.to_json(context).ok()?;
        match serde_json::from_value(json) {
            Ok(command) => Some(command),
            Err(err) => {
                error!("Failed to parse AwgenScript command: {}", err);
                None
            }
        }
    }
}
