//! This module contains the messages that can be received from the AwgenScript
//! engine.

use boa_engine::{Context, JsValue};
use serde::{Deserialize, Serialize};

/// The logic output enum represents all possible outputs that can be received
/// from the logic system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LogicCommands {
    /// A query to get the project settings.
    #[serde(rename = "get_project_settings")]
    GetProjectSettingsQuery,

    /// Update the project settings.
    #[serde(rename = "set_project_settings")]
    SetProjectSettings {
        /// The new name of the project.
        name: String,

        /// The new version of the project.
        version: String,
    },
}

impl LogicCommands {
    /// Converts the output into a JavaScript value, or returns `None` if the
    /// output cannot be converted.
    pub fn from_js_value(value: &JsValue, context: &mut Context) -> Option<Self> {
        let json = value.to_json(context).ok()?;
        serde_json::from_value(json).ok()
    }
}
