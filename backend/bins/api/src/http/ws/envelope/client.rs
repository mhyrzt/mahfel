use serde::{Deserialize, Serialize};

/// Frames accepted from a client.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    asyncapi_rust::schemars::JsonSchema,
    asyncapi_rust::ToAsyncApiMessage,
)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Authenticate the connection. `token` is validated by the authenticator;
    /// the interim development authenticator ignores it.
    Auth {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token: Option<String>,
    },
    /// Reply to the sender only.
    #[asyncapi(message_name = "ClientEcho")]
    Echo {
        text: String,
    },
    /// Fan out to every authenticated connection.
    #[asyncapi(message_name = "ClientBroadcast")]
    Broadcast {
        text: String,
    },
    /// Subscribe to a persistent room channel after the last applied event.
    SubscribeChat {
        room_id: String,
        after_sequence: i64,
    },
    UnsubscribeChat {
        room_id: String,
    },
    /// Short-lived indication; stops automatically without refresh.
    ChatTyping {
        room_id: String,
        active: bool,
    },
    /// A reconnect resolves to the same participant as an earlier join.
    Join {
        session_id: String,
    },
    Leave {},
    /// Mute another participant's media for one source.
    ///
    /// `kind` is "temporary" (the target may clear it) or "permanent" (only a
    /// moderator may). `source` defaults to the microphone.
    MuteParticipant {
        session_id: String,
        /// The target's actor id, not a connection: moderation resolves across
        /// all of a participant's connections.
        actor_id: String,
        kind: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
    },
    /// Clear a mute. A target may clear its own temporary mute; a permanent one
    /// needs a moderator.
    UnmuteParticipant {
        session_id: String,
        actor_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
    },
    /// End a participant's participation in this session. They may rejoin.
    RemoveParticipant {
        session_id: String,
        actor_id: String,
    },
    /// Bar a principal from a room or a session. Unlike removal, this holds.
    BlockPrincipal {
        /// A room block bars every session in that room; a session block is
        /// narrower. Exactly one of these is set.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        room_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        actor_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
        /// Unix seconds. Omitted means until explicitly lifted.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expires_at: Option<i64>,
    },
    UnblockPrincipal {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        room_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        actor_id: String,
    },
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
        /// "microphone" | "camera" | "screen". Omitted means the kind's obvious
        /// meaning: audio is a microphone and video is a camera. This selects
        /// which producers a moderation action applies to; it is never
        /// authorization.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
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
