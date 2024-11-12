//! This module implements the scripting engine and logic for the game. All
//! logic is received through the JavaScript runtime, which is then translated
//! into commands and executed on the game state.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use bevy::prelude::*;
use boa_engine::context::ContextBuilder;
use boa_engine::module::SimpleModuleLoader;
use boa_engine::{Context, Module, NativeFunction, Source, js_string};
use messages::{LogicInput, LogicOutput};
use queue::{ScriptEngineJobQueue, ScriptEngineShutdown};
use smol::LocalExecutor;
use smol::channel::{self, Receiver, Sender};

use crate::ui::GameState;

pub mod api;
pub mod messages;
pub mod queue;

/// The logic plugin is responsible for handling all game logic. This includes
/// the scripting engine, which is used to run the game's logic.
pub struct LogicPlugin;
impl Plugin for LogicPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_resource::<LogicPluginSettings>()
            .add_systems(OnEnter(GameState::Editor), begin_loop)
            .add_systems(
                OnExit(GameState::Editor),
                close_loop.run_if(resource_exists::<LogicMessageChannels>),
            );
    }
}

/// The logic plugin settings resource.
#[derive(Debug, Resource)]
pub struct LogicPluginSettings {
    /// The path to the script source folder.
    pub script_path: PathBuf,
}

impl Default for LogicPluginSettings {
    fn default() -> Self {
        Self {
            script_path: Path::new("./scripts").to_path_buf(),
        }
    }
}

/// The logic data resource contains the channels used to communicate with the
/// logic runtime.
#[derive(Debug, Resource)]
pub struct LogicMessageChannels {
    /// The channel to send messages to the logic system.
    send_channel: Sender<LogicInput>,

    /// The channel to receive messages from the logic system.
    receive_channel: Receiver<LogicOutput>,

    /// The shutdown listener signals the script engine to close.
    shutdown: ScriptEngineShutdown,
}

impl LogicMessageChannels {
    /// Sends a message to the logic system.
    ///
    /// If the channel is closed, this function does nothing.
    pub fn send(&self, message: LogicInput) {
        let _ = self.send_channel.try_send(message);
    }

    /// Receives a message from the logic system, or returns `None` if no
    /// message is available.
    ///
    /// If the channel is closed, this function returns `None`.
    pub fn receive(&self) -> Option<LogicOutput> {
        self.receive_channel.try_recv().ok()
    }

    /// Signals the logic system to shut down. If the logic system is already
    /// shut down, this function does nothing. Note that the script engine may
    /// not immediately stop, as it will wait for all running jobs to finish.
    pub fn shutdown(&self) {
        self.shutdown.shutdown();
    }
}

/// This system is called when the game state is set to the editor. It begins
/// the logic loop, which is responsible for running the game's logic in a
/// background thread.
fn begin_loop(settings: Res<LogicPluginSettings>, mut commands: Commands) {
    let (logic_in_send, logic_in_recv) = channel::unbounded();
    let (logic_out_send, logic_out_recv) = channel::unbounded();
    let shutdown = ScriptEngineShutdown::new();

    commands.insert_resource(LogicMessageChannels {
        send_channel: logic_in_send,
        receive_channel: logic_out_recv,
        shutdown: shutdown.clone(),
    });

    let script_path = settings.script_path.clone();
    std::thread::Builder::new()
        .name("ScriptEngine".to_string())
        .spawn(move || {
            logic_loop(script_path, logic_out_send, logic_in_recv, shutdown);
        })
        .unwrap();
}

/// This system is called when the game state is set to something other than the
/// editor. It closes the logic loop, which will stop the game's logic from
/// running. This will also remove the logic message channels resource.
fn close_loop(channels: Res<LogicMessageChannels>, mut commands: Commands) {
    channels.shutdown();
    commands.remove_resource::<LogicMessageChannels>();
}

/// The logic loop is a function that runs a JavaScript runtime and executes the
/// game's logic. It receives messages from the main Bevy systems and sends
/// messages back to them to execute commands.
fn logic_loop(
    path: PathBuf,
    send: Sender<LogicOutput>,
    receive: Receiver<LogicInput>,
    shutdown: ScriptEngineShutdown,
) {
    let executor = LocalExecutor::new();
    let queue = ScriptEngineJobQueue::new(executor, shutdown);
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
    register(c, "NATIVE_QUERY", 0, api::channels::build_query(receive));
    register(c, "NATIVE_SEND", 1, api::channels::build_send(send));

    let main_file = path.clone().canonicalize().unwrap().join("main.mjs");
    let relative_path = Path::new("./main.mjs");
    let file_reader = BufReader::new(File::open(&main_file).unwrap());
    let source = Source::from_reader(file_reader, Some(relative_path));
    let module = Module::parse(source, None, &mut context).unwrap();
    module_loader.insert(main_file, module.clone());

    module.load_link_evaluate(&mut context);
    context.run_jobs();
}

/// Registers a native function with the given name and argument count to the
/// script engine.
fn register(context: &mut Context, name: &str, args: usize, func: NativeFunction) {
    context
        .register_global_builtin_callable(js_string!(name), args, func)
        .unwrap();
}
