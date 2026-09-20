use thiserror::Error;

use super::{ClientMessage, Envelope, PROTOCOL_VERSION, ServerMessage};

/// Fallback used only if serializing a server message ever fails; server
/// messages contain only owned strings, so this is unreachable in practice.
const SERIALIZE_FALLBACK: &str =
    r#"{"v":1,"type":"error","code":"internal","message":"internal error"}"#;

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
