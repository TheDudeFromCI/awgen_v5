//! This module contains the commands that can be received from the AwgenScript
//! engine.

use boa_engine::{Context, JsValue};
use serde::{Deserialize, Serialize};

use super::queries::LogicQuery;

/// An enum that represents all possible commands that can be received from the
/// AwgenScript engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum LogicCommands {
    /// A command that is used to query the engine for information.
    Query {
        /// The query to be made.
        query: LogicQuery,
    },

    /// A command that is used to update the project settings.
    SetProjectSettings {
        /// The new name of the project.
        name: String,

        /// The new version of the project.
        version: String,
    },
}

impl LogicCommands {
    /// Converts the given [`JsValue`] into a [`LogicCommands`] instance, if
    /// possible. Returns `None` if the conversion fails.
    pub fn from_js_value(value: &JsValue, context: &mut Context) -> Option<Self> {
        let json = value.to_json(context).ok()?;
        serde_json::from_value(json).ok()
    }
}
