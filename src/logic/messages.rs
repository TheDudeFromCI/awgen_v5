//! This module contains the messages that can be sent to and received from the
//! logic system.

use boa_engine::JsValue;

/// The logic input enum represents all possible inputs that can be sent to the
/// logic system.
pub enum LogicInput {}

impl LogicInput {
    /// Converts the input into a JavaScript value.
    pub fn into_js_value(self) -> JsValue {
        match self {}
    }
}

/// The logic output enum represents all possible outputs that can be received
/// from the logic system.
pub enum LogicOutput {}

impl LogicOutput {
    /// Converts the output into a JavaScript value, or returns `None` if the
    /// output cannot be converted.
    pub fn from_js_value(_value: &JsValue) -> Option<Self> {
        None
    }
}
