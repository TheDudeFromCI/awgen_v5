//! This module contains the native API functions that are exposed to the
//! JavaScript code.

use std::future::Future;
use std::time::Duration;

use bevy::log::info;
use boa_engine::{Context, JsArgs, JsResult, JsValue};

/// This module implements the message channels that are used to communicate
/// between the logic system and the game engine.
pub mod channels {
    use std::future::Future;

    use boa_engine::{Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction};
    use smol::channel::{Receiver, Sender};

    use crate::logic::channels::{AwgenScriptReceiveChannel, AwgenScriptSendChannel};
    use crate::logic::commands::LogicCommands;
    use crate::logic::events::LogicEvent;

    /// Builds a native function that listens for the next input to the logic
    /// system. Note that action will assign a global variable to the receiver,
    /// which will be used to receive messages from the logic system.
    ///
    /// Calling this function more than once will overwrite the previous global
    /// receiver. The previous receiver will be closed.
    pub fn build_receive(receiver: Receiver<LogicEvent>) -> NativeFunction {
        /// The inner function that is called when the query function is
        /// invoked.
        fn inner(
            _this: &JsValue,
            _args: &[JsValue],
            _context: &mut Context,
        ) -> impl Future<Output = JsResult<JsValue>> {
            async move {
                let Some(message) = AwgenScriptReceiveChannel::recv().await else {
                    return Err(JsNativeError::error()
                        .with_message("RECEIVE message channel has been closed.")
                        .into());
                };

                Ok(JsValue::String(message.json().into()))
            }
        }

        AwgenScriptReceiveChannel::set(receiver);
        NativeFunction::from_async_fn(inner)
    }

    /// Builds a native function that sends a message from the logic system to
    /// Bevy. Note that action will assign a global variable to the sender,
    /// which will be used to send messages from the logic system.
    ///
    /// Calling this function more than once will overwrite the previous global
    /// receiver. The previous receiver will be closed.
    pub fn build_send(sender: Sender<LogicCommands>) -> NativeFunction {
        /// The inner function that is called when the send function is invoked.
        fn inner(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
            let message = LogicCommands::from_js_value(args.get_or_undefined(0), context);

            let Some(message) = message else {
                return Err(JsNativeError::error()
                    .with_message("Invalid message.")
                    .into());
            };

            if !AwgenScriptSendChannel::send(message) {
                return Err(JsNativeError::error()
                    .with_message("SEND message channel has been closed.")
                    .into());
            }

            Ok(JsValue::undefined())
        }

        AwgenScriptSendChannel::set(sender);
        NativeFunction::from_fn_ptr(inner)
    }
}

/// A native function that sleeps for a given number of milliseconds.
pub fn sleep(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> impl Future<Output = JsResult<JsValue>> {
    let millis = args.get_or_undefined(0).to_i32(context).unwrap_or(0).max(0) as u64;

    async move {
        smol::Timer::after(Duration::from_millis(millis)).await;
        Ok(JsValue::undefined())
    }
}

/// A native function that prints a message to the console.
pub fn print(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let message = args.get_or_undefined(0).to_string(context)?;
    info!("{}", message.to_std_string_escaped());
    Ok(JsValue::undefined())
}
