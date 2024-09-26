//! Block face rotation enum.

use serde::{Deserialize, Serialize};

/// The texture rotation of a face of a block.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaceRotation {
    /// No rotation.
    #[default]
    C0,

    /// 90 degrees clockwise rotation.
    C90,

    /// 180 degrees clockwise rotation.
    C180,

    /// 270 degrees clockwise rotation.
    C270,
}
