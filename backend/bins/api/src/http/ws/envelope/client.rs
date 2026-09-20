use serde::{Deserialize, Serialize};

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
