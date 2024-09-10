use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::WindowMode;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Enable debug mode
    #[arg(long)]
    debug: bool,

    /// Enable development mode
    #[arg(long)]
    dev: bool,

    /// The project workspace to open. If not provided, the current directory is
    /// used.
    #[arg(long)]
    project: Option<String>,
}

fn main() {
    let args = Args::parse();

    #[cfg(feature = "editor")]
    let dev = args.dev;
    #[cfg(not(feature = "editor"))]
    let dev = false;

    let name = "Unnamed Game";
    let title = match (dev, args.debug) {
        (true, true) => format!("Awgen Editor [{}] (debug)", name),
        (true, false) => format!("Awgen Editor [{}]", name),
        (false, true) => format!("{} (debug)", name),
        (false, false) => name.to_string(),
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title,
                        mode: WindowMode::Fullscreen,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: if args.debug {
                        bevy::log::Level::DEBUG
                    } else {
                        bevy::log::Level::INFO
                    },
                    ..default()
                }),
        )
        .run();
}
