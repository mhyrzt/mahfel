use serde::{Deserialize, Serialize};

/// Protocol version carried by every frame. Reject unknown versions.
pub const PROTOCOL_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope<T> {
    #[serde(default = "protocol_version")]
    pub v: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(flatten)]
    pub message: T,
}

// Used by `#[serde(default = ...)]` above.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn protocol_version() -> u8 {
    PROTOCOL_VERSION
}

impl<T> Envelope<T> {
    pub fn new(message: T) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            id: None,
            message,
        }
    }
}
