#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::path::PathBuf;

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::WindowMode;
use clap::Parser;
use settings::ProjectSettings;

mod camera;
mod settings;
mod ui;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,

    /// The project workspace to open. If not provided, the current directory is
    /// used.
    #[arg(short, long)]
    project: Option<String>,

    /// Launch the engine in fullscreen mode.
    #[arg(short, long)]
    fullscreen: bool,
}

/// Whether the engine is running in development mode.
pub const DEV_MODE: bool = cfg!(feature = "editor");

/// The key used to store the project name in the settings file.
pub const PROJECT_NAME_KEY: &str = "NAME";

/// The key used to store the project version in the settings file.
pub const PROJECT_VERSION_KEY: &str = "VERSION";

fn main() {
    let args = Args::parse();

    let Ok(cwd) = std::env::current_dir() else {
        eprintln!("Failed to get current directory.");
        std::process::exit(1);
    };

    let project_folder: PathBuf = match args.project {
        Some(path) => path.into(),
        None => cwd,
    };

    let settings = match ProjectSettings::new(project_folder, DEV_MODE) {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("Failed to open project settings: {err}");
            std::process::exit(1);
        }
    };

    let proj_name = match settings.get(PROJECT_NAME_KEY) {
        Ok(Some(name)) => name,
        Ok(None) => "Untitled".to_string(),
        Err(err) => {
            eprintln!("Failed to read project settings: {err}");
            std::process::exit(1);
        }
    };

    let proj_version = match settings.get(PROJECT_VERSION_KEY) {
        Ok(Some(version)) => version,
        Ok(None) => "0.0.1".to_string(),
        Err(err) => {
            eprintln!("Failed to read project settings: {err}");
            std::process::exit(1);
        }
    };

    let title = match (DEV_MODE, args.debug) {
        (true, true) => format!("Awgen Editor [{} - {}] (debug)", proj_name, proj_version),
        (true, false) => format!("Awgen Editor [{} - {}]", proj_name, proj_version),
        (false, true) => format!("{} - {} (debug)", proj_name, proj_version),
        (false, false) => format!("{} - {}", proj_name, proj_version),
    };

    let log_level = if args.debug {
        bevy::log::Level::DEBUG
    } else {
        bevy::log::Level::INFO
    };

    let window_mode = if args.fullscreen {
        WindowMode::Fullscreen
    } else {
        WindowMode::Windowed
    };

    App::new()
        .insert_resource(settings)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title,
                        mode: window_mode,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: log_level,
                    ..default()
                }),
        )
        .add_plugins(camera::CameraPlugin)
        .add_plugins(ui::menu::MainMenuPlugin)
        .run();
}
