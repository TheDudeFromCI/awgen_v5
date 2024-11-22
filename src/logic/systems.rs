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
use smol::channel::{Receiver, Sender};

use super::commands::LogicCommands;
use super::events::LogicEvent;
use super::queue::{ScriptEngineJobQueue, ScriptEngineShutdown};
use super::resources::AwgenScriptChannels;
use super::{LogicPluginSettings, api};
use crate::settings::ProjectSettings;
use crate::{PROJECT_NAME_DEFAULT, PROJECT_NAME_KEY, PROJECT_VERSION_DEFAULT, PROJECT_VERSION_KEY};

/// Handles the logic input channels.
pub fn handle_logic_outputs(
    project_settings: Res<ProjectSettings>,
    mut channels: ResMut<AwgenScriptChannels>,
) {
    while let Some(output) = channels.receive() {
        match output {
            LogicCommands::GetProjectSettingsQuery => {
                debug!("Received project settings query.");

                let name = project_settings
                    .get(PROJECT_NAME_KEY)
                    .unwrap()
                    .unwrap_or_else(|| PROJECT_NAME_DEFAULT.to_string());

                let version = project_settings
                    .get(PROJECT_VERSION_KEY)
                    .unwrap()
                    .unwrap_or_else(|| PROJECT_VERSION_DEFAULT.to_string());

                channels.send(LogicEvent::ProjectSettings { name, version });
            }

            LogicCommands::SetProjectSettings { name, version } => {
                info!(
                    "Updating project settings: name = {}, version = {}",
                    name, version
                );

                project_settings.set(PROJECT_NAME_KEY, &name).unwrap();
                project_settings.set(PROJECT_VERSION_KEY, &version).unwrap();
            }
        }
    }
}

/// This system creates the AwgenScript editor engine thread and initializes the
/// channels for communication between the engine and the main game loop.
#[cfg(feature = "editor")]
pub fn begin_editor_loop(
    settings: Res<LogicPluginSettings>,
    mut channels: ResMut<AwgenScriptChannels>,
) {
    begin_loop(
        settings.editor_script_path.clone(),
        "ScriptEngine-Editor".to_string(),
        &mut channels,
    );
}

/// This system creates the AwgenScript runtime engine thread and initializes
/// the channels for communication between the engine and the main game loop.
pub fn begin_runtime_loop(
    settings: Res<LogicPluginSettings>,
    mut channels: ResMut<AwgenScriptChannels>,
) {
    begin_loop(
        settings.runtime_script_path.clone(),
        "ScriptEngine-Runtime".to_string(),
        &mut channels,
    );
}

/// This function creates a new thread for the AwgenScript engine and
/// initializes the channels for communication between the engine and the main
/// game loop.
fn begin_loop(
    script_path: PathBuf,
    thread_name: String,
    channels: &mut ResMut<AwgenScriptChannels>,
) {
    let (in_send, in_recv) = smol::channel::unbounded();
    let (out_send, out_recv) = smol::channel::unbounded();
    let shutdown = ScriptEngineShutdown::new();
    channels.set_channels(in_send, out_recv, shutdown.clone());

    std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            exec_engine(script_path, out_send, in_recv, shutdown);
        })
        .unwrap();

    channels.send(LogicEvent::EngineStarted);
}

/// This system closes the active AwgenScript engine thread.
pub fn close_engine_loop(mut channels: ResMut<AwgenScriptChannels>) {
    channels.shutdown();
}

/// The logic loop is a function that runs a JavaScript runtime and executes the
/// game's logic. It receives messages from the main Bevy systems and sends
/// messages back to them to execute commands.
pub fn exec_engine(
    path: PathBuf,
    send: Sender<LogicCommands>,
    receive: Receiver<LogicEvent>,
    shutdown: ScriptEngineShutdown,
) {
    let queue = ScriptEngineJobQueue::new(shutdown);
    let module_loader = Rc::new(SimpleModuleLoader::new(path.clone()).unwrap());

    let mut context = ContextBuilder::new()
        .job_queue(Rc::new(queue))
        .module_loader(module_loader.clone())
        .build()
        .unwrap();

    let c = &mut context;
    let fn_ptr = NativeFunction::from_fn_ptr;
    let async_fn_ptr = NativeFunction::from_async_fn;

    register(c, "print", 1, fn_ptr(api::print));
    register(c, "sleep", 1, async_fn_ptr(api::sleep));
    register(c, "NATIVE_QUERY", 0, api::channels::build_receive(receive));
    register(c, "NATIVE_SEND", 1, api::channels::build_send(send));

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
