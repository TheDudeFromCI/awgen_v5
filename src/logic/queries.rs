//! This module contains the definition of the queries that can be made from the
//! AwgenScript engine to the main game.

use core::fmt;

use serde::{Deserialize, Serialize};

/// An enum that represents all possible queries that can be made from the
/// AwgenScript engine, and their corresponding responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "query", rename_all = "snake_case")]
pub enum LogicQuery {
    /// A query to get the current project settings.
    ProjectSettings,
}

impl fmt::Display for LogicQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicQuery::ProjectSettings => write!(f, "project_settings"),
        }
    }
}
