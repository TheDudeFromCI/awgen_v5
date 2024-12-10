//! This module implements the systems for the logic plugin.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use bevy::prelude::*;
use boa_engine::builtins::promise::PromiseState;
use boa_engine::context::ContextBuilder;
use boa_engine::module::SimpleModuleLoader;
use boa_engine::{Context, JsError, Module, NativeFunction, Source, js_string};

use super::channels::{AwgenScriptReceiveChannel, AwgenScriptSendChannel};
use super::commands::LogicCommands;
use super::events::LogicEvent;
use super::queue::{ScriptEngineJobQueue, ScriptEngineShutdown};
use super::resources::AwgenScriptChannels;
use super::{LogicPluginSettings, api};
use crate::blocks::tileset::TilesetDefinition;
use crate::logic::commands::EditTilesetAction;
use crate::settings::ProjectSettings;
use crate::{PROJECT_NAME_DEFAULT, PROJECT_NAME_KEY, PROJECT_VERSION_DEFAULT, PROJECT_VERSION_KEY};

/// Handles the logic input channels.
pub fn handle_logic_outputs(
    project_settings: Res<ProjectSettings>,
    mut channels: ResMut<AwgenScriptChannels>,
) {
    while let Some(output) = channels.receive() {
        match output {
            LogicCommands::SetProjectName { name } => {
                info!("Updating project name: {}", name);
                project_settings.set(PROJECT_NAME_KEY, Some(&name)).unwrap();
            }

            LogicCommands::SetProjectVersion { version } => {
                info!("Updating project version: {}", version);
                project_settings
                    .set(PROJECT_VERSION_KEY, Some(&version))
                    .unwrap();
            }

            LogicCommands::EditTileset { uuid, action } => match action {
                EditTilesetAction::Create { name } => {
                    info!("Creating new tileset {}", uuid);
                    debug!("Name: {}", name);

                    let definition = TilesetDefinition { uuid, name };
                    project_settings.update_tileset(&definition).unwrap();
                }
                EditTilesetAction::Update { name } => {
                    info!("Updating tileset {}", uuid);
                    debug!("New name: {}", name);

                    let definition = TilesetDefinition { uuid, name };
                    project_settings.update_tileset(&definition).unwrap();
                }
                EditTilesetAction::Delete => {
                    info!("Deleting tileset {}", uuid);
                    project_settings.remove_tileset(&uuid).unwrap();
                }
            },
        }
    }
}

/// This system creates the AwgenScript editor engine thread and initializes the
/// channels for communication between the engine and the main game loop.
#[cfg(feature = "editor")]
pub fn begin_editor_loop(
    settings: Res<LogicPluginSettings>,
    project_settings: Res<ProjectSettings>,
    mut channels: ResMut<AwgenScriptChannels>,
) {
    begin_loop(
        settings.editor_script_path.clone(),
        "ScriptEngine-Editor".to_string(),
        &project_settings,
        &mut channels,
    );
}

/// This system creates the AwgenScript runtime engine thread and initializes
/// the channels for communication between the engine and the main game loop.
pub fn begin_runtime_loop(
    settings: Res<LogicPluginSettings>,
    project_settings: Res<ProjectSettings>,
    mut channels: ResMut<AwgenScriptChannels>,
) {
    begin_loop(
        settings.runtime_script_path.clone(),
        "ScriptEngine-Runtime".to_string(),
        &project_settings,
        &mut channels,
    );
}

/// This function creates a new thread for the AwgenScript engine and
/// initializes the channels for communication between the engine and the main
/// game loop.
fn begin_loop(
    script_path: PathBuf,
    thread_name: String,
    project_settings: &Res<ProjectSettings>,
    channels: &mut ResMut<AwgenScriptChannels>,
) {
    let (in_send, in_recv) = smol::channel::unbounded();
    let (out_send, out_recv) = smol::channel::unbounded();
    let shutdown = ScriptEngineShutdown::new();
    channels.set_channels(in_send, out_recv, shutdown.clone());

    std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            AwgenScriptReceiveChannel::set(in_recv);
            AwgenScriptSendChannel::set(out_send);
            exec_engine(script_path, shutdown);
        })
        .unwrap();

    channels.send(LogicEvent::EngineStarted {
        project_name: project_settings
            .get(PROJECT_NAME_KEY)
            .unwrap()
            .unwrap_or_else(|| PROJECT_NAME_DEFAULT.to_string()),
        project_version: project_settings
            .get(PROJECT_VERSION_KEY)
            .unwrap()
            .unwrap_or_else(|| PROJECT_VERSION_DEFAULT.to_string()),
        tilesets: project_settings.list_tilesets().unwrap(),
    });
}

/// This system closes the active AwgenScript engine thread.
pub fn close_engine_loop(mut channels: ResMut<AwgenScriptChannels>) {
    channels.shutdown();
}

/// The logic loop is a function that runs a JavaScript runtime and executes the
/// game's logic. It receives messages from the main Bevy systems and sends
/// messages back to them to execute commands.
pub fn exec_engine(path: PathBuf, shutdown: ScriptEngineShutdown) {
    let queue = ScriptEngineJobQueue::new(shutdown);
    let module_loader = Rc::new(SimpleModuleLoader::new(path.clone()).unwrap());

    let mut context = ContextBuilder::new()
        .job_queue(Rc::new(queue))
        .module_loader(module_loader.clone())
        .build()
        .unwrap();

    let c = &mut context;
    register(c, "print", 1, NativeFunction::from_fn_ptr(api::print));
    register(c, "sleep", 1, NativeFunction::from_async_fn(api::sleep));
    register(c, "UUID", 0, NativeFunction::from_fn_ptr(api::uuid));
    register(c, "EVENT", 0, NativeFunction::from_async_fn(api::event));
    register(c, "COMMAND", 1, NativeFunction::from_fn_ptr(api::command));

    let main_file = path.clone().canonicalize().unwrap().join("main.mjs");
    let relative_path = Path::new("./main.mjs");
    let file_reader = BufReader::new(File::open(&main_file).unwrap());
    let source = Source::from_reader(file_reader, Some(relative_path));
    let module = Module::parse(source, None, &mut context).unwrap();
    module_loader.insert(main_file, module.clone());

    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs();

    match promise.state() {
        PromiseState::Pending => error!("Failed to execute all AwgenScript jobs."),
        PromiseState::Fulfilled(_) => {}
        PromiseState::Rejected(err) => {
            error!(
                "AwgenScript exited with an error: {:?}",
                JsError::from_opaque(err).try_native(&mut context).unwrap()
            );
        }
    }
}

/// Registers a native function with the given name and argument count to the
/// script engine.
fn register(context: &mut Context, name: &str, args: usize, func: NativeFunction) {
    context
        .register_global_builtin_callable(js_string!(name), args, func)
        .unwrap();
}
