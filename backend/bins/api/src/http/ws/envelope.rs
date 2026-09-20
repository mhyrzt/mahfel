//! Versioned JSON envelope for the realtime gateway.
//!
//! Every frame is `{ "v": <version>, "type": <kind>, ... }`. The version is
//! independent of the URL so the protocol can evolve without a new route.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Protocol version carried by every frame. Reject unknown versions.
pub const PROTOCOL_VERSION: u8 = 1;

/// Fallback used only if serializing a server message ever fails; server
/// messages contain only owned strings, so this is unreachable in practice.
const SERIALIZE_FALLBACK: &str =
    r#"{"v":1,"type":"error","code":"internal","message":"internal error"}"#;

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

/// Frames accepted from a client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Authenticate the connection. `token` is validated by the authenticator;
    /// the interim development authenticator ignores it.
    Auth {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token: Option<String>,
    },
    /// Reply to the sender only.
    Echo {
        text: String,
    },
    /// Fan out to every authenticated connection.
    Broadcast {
        text: String,
    },
    // --- RTC signaling (M3-06) ---
    CreateTransport {
        room_id: String,
    },
    ConnectTransport {
        room_id: String,
        transport_id: String,
        dtls_parameters: String,
    },
    Produce {
        room_id: String,
        transport_id: String,
        kind: String,
        rtp_parameters: String,
    },
    Consume {
        room_id: String,
        transport_id: String,
        producer_id: String,
        rtp_capabilities: String,
    },
    PauseConsumer {
        room_id: String,
        consumer_id: String,
    },
    ResumeConsumer {
        room_id: String,
        consumer_id: String,
    },
}

/// Frames sent by the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome {
        connection_id: String,
    },
    Echo {
        text: String,
    },
    Broadcast {
        from: String,
        text: String,
    },
    Error {
        code: String,
        message: String,
    },
    Bye {
        reason: String,
    },
    // --- RTC responses ---
    TransportCreated {
        room_id: String,
        transport_id: String,
        ice_parameters: String,
        ice_candidates: String,
        dtls_parameters: String,
    },
    TransportConnected {
        room_id: String,
        transport_id: String,
    },
    Produced {
        room_id: String,
        producer_id: String,
    },
    Consumed {
        room_id: String,
        consumer_id: String,
        rtp_parameters: String,
        paused: bool,
    },
    ConsumerPaused {
        room_id: String,
        consumer_id: String,
    },
    ConsumerResumed {
        room_id: String,
        consumer_id: String,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("malformed message")]
    Malformed,
    #[error("unsupported protocol version {0}")]
    UnsupportedVersion(u8),
}

/// Parse a client frame, rejecting unknown protocol versions.
pub fn parse(text: &str) -> Result<ClientMessage, ProtocolError> {
    parse_envelope(text).map(|env| env.message)
}

/// Parse a client frame preserving the envelope (including `id`).
pub fn parse_envelope(text: &str) -> Result<Envelope<ClientMessage>, ProtocolError> {
    let envelope: Envelope<ClientMessage> =
        serde_json::from_str(text).map_err(|_| ProtocolError::Malformed)?;
    if envelope.v != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(envelope.v));
    }
    Ok(envelope)
}

/// Encode a server frame into the versioned envelope.
#[must_use]
pub fn encode(message: ServerMessage) -> String {
    serde_json::to_string(&Envelope::new(message)).unwrap_or_else(|_| SERIALIZE_FALLBACK.to_owned())
}

/// Encode a server frame preserving a request id for correlation.
#[must_use]
pub fn encode_with_id(id: Option<String>, message: ServerMessage) -> String {
    let envelope = Envelope {
        v: PROTOCOL_VERSION,
        id,
        message,
    };
    serde_json::to_string(&envelope).unwrap_or_else(|_| SERIALIZE_FALLBACK.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_frame_round_trips() {
        let text = r#"{"v":1,"type":"echo","text":"hi"}"#;
        assert_eq!(
            parse(text).expect("valid frame"),
            ClientMessage::Echo {
                text: "hi".to_owned()
            }
        );
    }

    #[test]
    fn version_defaults_to_current() {
        assert_eq!(
            parse(r#"{"type":"echo","text":"hi"}"#).expect("valid frame"),
            ClientMessage::Echo {
                text: "hi".to_owned()
            }
        );
    }

    #[test]
    fn future_version_is_rejected() {
        assert_eq!(
            parse(r#"{"v":2,"type":"echo","text":"hi"}"#).expect_err("unsupported"),
            ProtocolError::UnsupportedVersion(2)
        );
    }

    #[test]
    fn unknown_type_is_malformed() {
        assert_eq!(
            parse(r#"{"v":1,"type":"nope"}"#).expect_err("malformed"),
            ProtocolError::Malformed
        );
    }

    #[test]
    fn server_frame_carries_version_and_type() {
        let encoded = encode(ServerMessage::Welcome {
            connection_id: "abc".to_owned(),
        });
        assert!(encoded.contains(r#""v":1"#));
        assert!(encoded.contains(r#""type":"welcome""#));
    }
}
