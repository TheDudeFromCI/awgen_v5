//! This module contains all the events that can be sent to the AwgenScript
//! engine.

use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

/// An enum that represents all possible events that can be sent to the
/// AwgenScript engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum LogicEvent {
    /// An event that is triggered once when the engine is first started.
    EngineStarted,

    /// An event that is triggered in response to a query from the AwgenScript
    /// engine.
    QueryResponse {
        /// The query response data.
        data: HashMap<String, String>,
    },
}

impl LogicEvent {
    /// Converts the input into a JSON string.
    pub fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
