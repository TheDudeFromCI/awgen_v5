//! This module implements a handler for reading and writing project settings in
//! an SQLite database.

use std::path::PathBuf;

use bevy::prelude::*;
use sqlite::{Connection, ConnectionThreadSafe, OpenFlags};
use uuid::Uuid;

use crate::blocks::tileset::TilesetDefinition;

/// This resource contains connection access to the project settings file.
#[derive(Resource)]
pub struct ProjectSettings {
    /// The SQLite connection to the project settings file.
    connection: ConnectionThreadSafe,
}

impl ProjectSettings {
    /// Creates a new instance of the project settings resource.
    ///
    /// If `create` is `true`, the settings file will be created if it does not
    /// exist. If false, an error will be returned if the file does not exist.
    pub fn new(
        project_folder: impl Into<PathBuf>,
        create: bool,
    ) -> Result<Self, ProjectSettingsError> {
        let project_folder = project_folder.into();
        let settings_file = project_folder.join("settings.awgen");

        let mut flags = OpenFlags::new().with_read_write();

        if create {
            flags = flags.with_create();
        }

        let connection = Connection::open_thread_safe_with_flags(settings_file, flags)
            .map_err(ProjectSettingsError::Io)?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
        )?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS tilesets (
                uuid TEXT PRIMARY KEY,
                name TEXT
            )",
        )?;

        Ok(Self { connection })
    }

    /// Gets a property from the project settings. Returns `None` if the
    /// property does not exist. An error is returned if an SQL error occurs.
    pub fn get(&self, key: &str) -> Result<Option<String>, ProjectSettingsError> {
        let mut statement = self.connection.prepare(
            "SELECT value FROM settings
             WHERE key = :key",
        )?;
        statement.bind((":key", key))?;

        if statement.next()? != sqlite::State::Row {
            return Ok(None);
        }

        Ok(Some(statement.read::<String, _>("value")?))
    }

    /// Sets a property in the project settings. If the property already exists,
    /// it will be updated. An error is returned if an SQL error occurs.
    ///
    /// If the value is set to `None`, the property will be deleted.
    pub fn set(&self, key: &str, value: Option<&str>) -> Result<(), ProjectSettingsError> {
        let mut statement;

        if let Some(value) = value {
            statement = self.connection.prepare(
                "INSERT OR REPLACE INTO settings (key, value)
                 VALUES (:key, :value)",
            )?;
            statement.bind((":key", key))?;
            statement.bind((":value", value))?;
        } else {
            statement = self.connection.prepare(
                "DELETE FROM settings
                 WHERE key = :key",
            )?;
            statement.bind((":key", key))?;
        }

        statement.next()?;
        Ok(())
    }

    /// Gets a list of all tilesets in the project. An error is returned if an
    /// SQL error occurs.
    pub fn list_tilesets(&self) -> Result<Vec<TilesetDefinition>, ProjectSettingsError> {
        let mut statement = self.connection.prepare(
            "SELECT uuid, name
             FROM tilesets",
        )?;

        let mut tilesets = Vec::new();
        while statement.next()? == sqlite::State::Row {
            let uuid = statement.read::<String, _>("uuid")?;
            let name = statement.read::<String, _>("name")?;
            tilesets.push(TilesetDefinition {
                uuid: Uuid::parse_str(&uuid).unwrap(),
                name,
            });
        }

        Ok(tilesets)
    }

    /// Updates a tileset properties, creating a new tileset if needed. An error
    /// is returned if an SQL error
    pub fn update_tileset(&self, tileset: &TilesetDefinition) -> Result<(), ProjectSettingsError> {
        let mut statement = self.connection.prepare(
            "REPLACE INTO tilesets (uuid, name)
             VALUES (:uuid, :name)",
        )?;
        statement.bind((":uuid", tileset.uuid.to_string().as_str()))?;
        statement.bind((":name", tileset.name.as_str()))?;
        statement.next()?;
        Ok(())
    }

    /// Removes a tileset from the project. An error is returned if an SQL error
    /// occurs.
    pub fn remove_tileset(&self, uuid: &Uuid) -> Result<(), ProjectSettingsError> {
        let mut statement = self.connection.prepare(
            "DELETE FROM tilesets
             WHERE uuid = :uuid",
        )?;
        statement.bind((":uuid", uuid.to_string().as_str()))?;
        statement.next()?;
        Ok(())
    }
}

/// An error that can occur when working with project settings.
#[derive(Debug, thiserror::Error)]
pub enum ProjectSettingsError {
    /// The settings file could not be opened.
    #[error("The settings file could not be opened: {0}")]
    Io(#[source] sqlite::Error),

    /// An error occurred while executing a SQL query.
    #[error("An error occurred while executing a SQL query: {0}")]
    Sql(#[from] sqlite::Error),
}
