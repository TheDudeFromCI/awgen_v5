//! This module contains the global singleton channels for sending and receiving
//! messages between the AwgenScript engine and the main game.

use bevy::log::info;
use smol::channel::{Receiver, Sender};

use super::commands::LogicCommands;
use super::events::LogicEvent;

/// The global sender for logic messages.
static mut SENDER: Option<Sender<LogicCommands>> = None;

/// The global receiver for logic messages.
static mut RECEIVER: Option<Receiver<LogicEvent>> = None;

/// A singleton channel for sending messages from the AwgenScript engine to the
/// main game.
pub struct AwgenScriptSendChannel;
impl AwgenScriptSendChannel {
    /// Closes the global sender for logic messages, if it is open.
    pub fn close() {
        if let Some(sender) = unsafe { SENDER.as_ref() } {
            sender.close();
            unsafe { SENDER = None };
            info!("ScriptEngine input message channel closed.");
        }
    }

    /// Sets the global sender for logic messages, closing the previous sender
    /// if it exists, and replacing it.
    pub fn set(new_sender: Sender<LogicCommands>) {
        Self::close();
        unsafe { SENDER = Some(new_sender) };
        info!("ScriptEngine input message channel assigned.");
    }

    /// Sends a message to the global sender for logic messages. If the channel
    /// does not exist or is already closed, this function will return false.
    pub fn send(message: LogicCommands) -> bool {
        let Some(sender) = (unsafe { SENDER.as_ref() }) else {
            return false;
        };

        // Sending blocking messages is fine here as the channels are unbounded
        // and will always return immediately.
        if sender.send_blocking(message).is_err() {
            Self::close();
            return false;
        }

        true
    }
}

/// A singleton channel for receiving messages from the main game to the
/// AwgenScript engine.
pub struct AwgenScriptReceiveChannel;
impl AwgenScriptReceiveChannel {
    /// Closes the global receiver for logic messages, if it is open.
    pub fn close() {
        if let Some(receiver) = unsafe { RECEIVER.as_ref() } {
            receiver.close();
            unsafe { RECEIVER = None };
            info!("ScriptEngine output message channel closed.");
        }
    }

    /// Sets the global receiver for logic messages, closing the previous
    /// receiver if it exists, and replacing it.
    pub fn set(new_receiver: Receiver<LogicEvent>) {
        Self::close();
        unsafe { RECEIVER = Some(new_receiver) };
        info!("ScriptEngine output message channel assigned.");
    }

    /// Receives a message from the global receiver for logic messages. If the
    /// channel does not exist or is already closed, this function will return
    /// `None`.
    pub async fn recv() -> Option<LogicEvent> {
        let Some(receiver) = (unsafe { RECEIVER.as_ref() }) else {
            Self::close();
            return None;
        };

        let Ok(message) = receiver.recv().await else {
            Self::close();
            return None;
        };

        Some(message)
    }
}
