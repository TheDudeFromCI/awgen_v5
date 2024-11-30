//! This module contains the native API functions that are exposed to the
//! JavaScript code.

use std::future::Future;
use std::time::Duration;

use bevy::log::info;
use boa_engine::{Context, JsArgs, JsNativeError, JsResult, JsValue};

use crate::logic::channels::{AwgenScriptReceiveChannel, AwgenScriptSendChannel};
use crate::logic::commands::LogicCommands;

/// A native async function that listens for the next incoming event from the
/// main game.
pub fn event(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> impl Future<Output = JsResult<JsValue>> {
    async move {
        let Some(message) = AwgenScriptReceiveChannel::recv().await else {
            return Err(JsNativeError::error()
                .with_message("Event channel has been closed.")
                .into());
        };

        Ok(JsValue::String(message.json().into()))
    }
}

/// A native function that sends a command to the main game.
pub fn command(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
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
