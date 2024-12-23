#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::Termination;

use bevy::asset::io::AssetSourceBuilder;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_egui::EguiPlugin;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_mod_picking::DefaultPickingPlugins;
use clap::Parser;
use logic::LogicPluginSettings;
use settings::ProjectSettings;

mod blocks;
mod camera;
mod gamestate;
mod gizmos;
mod logic;
mod map;
mod math;
mod settings;
mod tools;
mod ui;
mod utilities;

/// The command line arguments definition for the engine.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,

    /// Enable trace logging (only available in development builds)
    /// Requires the `debug` flag to be enabled.
    #[cfg(debug_assertions)]
    #[arg(short, long)]
    trace: bool,

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

/// The default project name if none is provided.
pub const PROJECT_NAME_DEFAULT: &str = "Untitled";

/// The key used to store the project version in the settings file.
pub const PROJECT_VERSION_KEY: &str = "VERSION";

/// The default project version if none is provided.
pub const PROJECT_VERSION_DEFAULT: &str = "0.0.1";

/// The main function for the Awgen Engine.
fn main() -> impl Termination {
    let args = Args::parse();

    println!("Awgen Engine v{}", env!("CARGO_PKG_VERSION"));

    if DEV_MODE {
        println!("Running in development mode.");
    } else {
        println!("Running in player mode.");
    }

    let Ok(cwd) = std::env::current_dir() else {
        eprintln!("Failed to get current directory.");
        std::process::exit(1);
    };

    let project_folder: PathBuf = match args.project {
        Some(path) => path.into(),
        None => cwd,
    };

    let asset_folder = format!("{}/assets", project_folder.display());

    println!("Opening project at: {}", project_folder.display());

    let settings = match ProjectSettings::new(project_folder, DEV_MODE) {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("Failed to open project settings: {err}");
            std::process::exit(1);
        }
    };

    let proj_name = match settings.get(PROJECT_NAME_KEY) {
        Ok(Some(name)) => name,
        Ok(None) => PROJECT_NAME_DEFAULT.to_string(),
        Err(err) => {
            eprintln!("Failed to read project settings: {err}");
            std::process::exit(1);
        }
    };

    let proj_version = match settings.get(PROJECT_VERSION_KEY) {
        Ok(Some(version)) => version,
        Ok(None) => PROJECT_VERSION_DEFAULT.to_string(),
        Err(err) => {
            eprintln!("Failed to read project settings: {err}");
            std::process::exit(1);
        }
    };

    println!("Project name: {}", proj_name);
    println!("Project version: {}", proj_version);

    let title = match (DEV_MODE, args.debug) {
        (true, true) => format!("Awgen Editor [{} - {}] (debug)", proj_name, proj_version),
        (true, false) => format!("Awgen Editor [{} - {}]", proj_name, proj_version),
        (false, true) => format!("{} - {} (debug)", proj_name, proj_version),
        (false, false) => format!("{} - {}", proj_name, proj_version),
    };

    println!("Debug enabled: {}", args.debug);
    let log_level = if args.debug {
        #[cfg(debug_assertions)]
        if args.trace {
            println!("Trace logging enabled.");
            bevy::log::Level::TRACE
        } else {
            bevy::log::Level::DEBUG
        }
        #[cfg(not(debug_assertions))]
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
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(settings)
        .insert_resource(LogicPluginSettings {
            editor_script_path: "./assets/editor_scripts".into(),
            runtime_script_path: format!("{}/scripts", asset_folder).into(),
        })
        .register_asset_source(
            "editor",
            AssetSourceBuilder::platform_default("assets", None),
        )
        .register_asset_source(
            "project",
            AssetSourceBuilder::platform_default(&asset_folder, None),
        )
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
                    filter: "wgpu=error,naga=warn,calloop=debug,polling=debug".to_string(),
                    ..default()
                }),
        )
        .add_plugins((DefaultPickingPlugins, EguiPlugin, FramepacePlugin))
        .add_plugins((
            camera::CameraPlugin,
            ui::AwgenUIPlugin,
            blocks::BlocksPlugin,
            map::VoxelWorldPlugin,
            gizmos::GizmosPlugin,
            logic::LogicPlugin,
        ))
        .init_state::<gamestate::GameState>()
        .add_systems(Startup, |mut settings: ResMut<FramepaceSettings>| {
            settings.limiter = Limiter::from_framerate(60.0);
        })
        .add_systems(Startup, gamestate::to_splash_screen)
        .run()
}
