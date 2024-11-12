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

    use bevy::log::info;
    use boa_engine::{Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction};
    use smol::channel::{Receiver, Sender};

    use crate::logic::messages::{LogicInput, LogicOutput};

    /// The global receiver for logic messages.
    static mut RECEIVER: Option<Receiver<LogicInput>> = None;

    /// The global sender for logic messages.
    static mut SENDER: Option<Sender<LogicOutput>> = None;

    /// Returns the global sender for logic messages, or `None` if the sender is
    /// not set.
    #[inline(always)]
    fn get_sender() -> Option<&'static Sender<LogicOutput>> {
        unsafe { SENDER.as_ref() }
    }

    /// Returns the global receiver for logic messages, or `None` if the
    /// receiver is not set.
    #[inline(always)]
    fn get_receiver() -> Option<&'static Receiver<LogicInput>> {
        unsafe { RECEIVER.as_ref() }
    }

    /// Sets the global sender for logic messages, closing the previous sender
    /// if it exists.
    #[inline(always)]
    fn set_sender(new_sender: Sender<LogicOutput>) {
        close_sender();
        unsafe { SENDER = Some(new_sender) };
        info!("ScriptEngine output message channel assigned.");
    }

    /// Sets the global receiver for logic messages, closing the previous
    /// receiver if it exists.
    #[inline(always)]
    fn set_receiver(new_receiver: Receiver<LogicInput>) {
        close_receiver();
        unsafe { RECEIVER = Some(new_receiver) };
        info!("ScriptEngine input message channel assigned.");
    }

    /// Closes the global sender for logic messages.
    #[inline(always)]
    fn close_sender() {
        if let Some(sender) = get_sender() {
            sender.close();
            unsafe { SENDER = None };
            info!("ScriptEngine output message channel closed.");
        }
    }

    /// Closes the global receiver for logic messages.
    #[inline(always)]
    fn close_receiver() {
        if let Some(receiver) = get_receiver() {
            receiver.close();
            unsafe { RECEIVER = None };
            info!("ScriptEngine input message channel closed.");
        }
    }

    /// Builds a native function that queries the logic system for a message.
    /// Note that action will assign a global variable to the receiver,
    /// which will be used to receive messages from the logic system.
    ///
    /// Calling this function more than once will overwrite the previous global
    /// receiver. The previous receiver will be closed.
    pub fn build_query(receive: Receiver<LogicInput>) -> NativeFunction {
        /// The inner function that is called when the query function is
        /// invoked.
        fn inner(
            _this: &JsValue,
            _args: &[JsValue],
            _context: &mut Context,
        ) -> impl Future<Output = JsResult<JsValue>> {
            async move {
                let Some(receiver) = get_receiver() else {
                    return Err(JsNativeError::error()
                        .with_message("Input message channel is not open.")
                        .into());
                };

                let Ok(message) = receiver.recv().await else {
                    close_receiver();
                    return Err(JsNativeError::error()
                        .with_message("Input message channel has been closed.")
                        .into());
                };

                Ok(message.into_js_value())
            }
        }

        set_receiver(receive);
        NativeFunction::from_async_fn(inner)
    }

    /// Builds a native function that sends a message from the logic system to
    /// Bevy. Note that action will assign a global variable to the sender,
    /// which will be used to send messages from the logic system.
    ///
    /// Calling this function more than once will overwrite the previous global
    /// receiver. The previous receiver will be closed.
    pub fn build_send(sender: Sender<LogicOutput>) -> NativeFunction {
        /// The inner function that is called when the send function is invoked.
        fn inner(
            _this: &JsValue,
            args: &[JsValue],
            _context: &mut Context,
        ) -> impl Future<Output = JsResult<JsValue>> {
            let message = LogicOutput::from_js_value(args.get_or_undefined(0));

            async move {
                let Some(message) = message else {
                    return Err(JsNativeError::error()
                        .with_message("Invalid message.")
                        .into());
                };

                let Some(sender) = get_sender() else {
                    return Err(JsNativeError::error()
                        .with_message("Output message channel is not open.")
                        .into());
                };

                if sender.send(message).await.is_err() {
                    close_sender();
                    return Err(JsNativeError::error()
                        .with_message("Output message channel has been closed.")
                        .into());
                };

                Ok(JsValue::undefined())
            }
        }

        set_sender(sender);
        NativeFunction::from_async_fn(inner)
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
