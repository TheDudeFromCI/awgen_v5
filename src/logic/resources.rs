//! The resources module contains the resources used by the logic plugin.

use bevy::prelude::*;
use smol::channel::{Receiver, Sender};

use super::commands::LogicCommands;
use super::events::LogicEvent;
use super::queue::ScriptEngineShutdown;

/// The logic data resource contains the channels used to communicate with the
/// AwgenScript engine.
#[derive(Debug, Default, Resource)]
pub struct AwgenScriptChannels {
    /// The channel to send messages to the active AwgenScript engine. May be
    /// `None` if there is no engine running.
    send_channel: Option<Sender<LogicEvent>>,

    /// The channel to receive messages from the active AwgenScript engine. May
    /// be `None` if there is no engine running.
    receive_channel: Option<Receiver<LogicCommands>>,

    /// The signal for the active AwgenScript  engine to shut down. May be
    /// `None` if there is no engine running.
    shutdown: Option<ScriptEngineShutdown>,
}

impl AwgenScriptChannels {
    /// Assigns the channels, closing and replacing the previous channels if
    /// they exist.
    pub fn set_channels(
        &mut self,
        send_channel: Sender<LogicEvent>,
        receive_channel: Receiver<LogicCommands>,
        shutdown: ScriptEngineShutdown,
    ) {
        self.shutdown();
        self.send_channel = Some(send_channel);
        self.receive_channel = Some(receive_channel);
        self.shutdown = Some(shutdown);
    }

    /// Sends a message to the active AwgenScript engine.
    ///
    /// If the channel is closed, this function does nothing.
    pub fn send(&self, message: LogicEvent) {
        if let Some(channel) = &self.send_channel {
            if let Err(e) = channel.try_send(message) {
                error!("Failed to send message to AwgenScript engine: {}", e);
            }
        }
    }

    /// Receives a message from the active AwgenScript engine, or returns
    /// `None` if no message is available.
    ///
    /// If the channel is closed, this function returns `None`.
    pub fn receive(&mut self) -> Option<LogicCommands> {
        if let Some(channel) = &self.receive_channel {
            channel.try_recv().ok()
        } else {
            None
        }
    }

    /// Signals the active AwgenScript engine to shut down. This function does
    /// nothing if there is no active engines.
    pub fn shutdown(&mut self) {
        if let Some(shutdown) = &self.shutdown {
            info!("Shutting down AwgenScript engine.");

            shutdown.shutdown();
            self.shutdown = None;
        }

        if let Some(channel) = &self.send_channel {
            debug!("Closing AwgenScript engine send channel.");
            let _ = channel.close();
            self.send_channel = None;
        }

        if let Some(channel) = &self.receive_channel {
            debug!("Closing AwgenScript engine receive channel.");
            let _ = channel.close();
            self.receive_channel = None;
        }
    }
}
