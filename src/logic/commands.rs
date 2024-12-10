//! This module contains the commands that can be received from the AwgenScript
//! engine.

use bevy::log::error;
use boa_engine::{Context, JsValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

    /// A command that is used to edit a tileset.
    EditTileset {
        /// The uuid of the tileset to modify.
        uuid: Uuid,

        /// The action to take on the tileset.
        action: EditTilesetAction,
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

/// An enum that represents all possible actions that can be taken on a tileset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum EditTilesetAction {
    /// Creates a new tileset with the given properties.
    Create {
        /// The name of the new tileset.
        name: String,
    },

    /// Updates the properties of the tileset.
    Update {
        /// The new name of the tileset.
        name: String,
    },

    /// Deletes the tileset.
    Delete,
}
