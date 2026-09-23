use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, asyncapi_rust::schemars::JsonSchema,
)]
pub struct ChatEventPayload {
    pub room_id: String,
    pub sequence: i64,
    pub event_id: String,
    pub kind: String,
    pub message_id: String,
    pub parent_id: Option<String>,
    pub author_id: String,
    pub reaction_actor_id: Option<String>,
    pub reaction_emoji: Option<String>,
    pub content: Option<String>,
    pub created_at: String,
    pub edited_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// Frames sent by the server.
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
    ChatSubscribed {
        room_id: String,
        head_sequence: i64,
    },
    ChatActivity {
        room_id: String,
        online_count: u64,
        typing_count: u64,
    },
    ChatEvent {
        event: Box<ChatEventPayload>,
    },
    Error {
        code: String,
        message: String,
    },
    Bye {
        reason: String,
    },
    /// A reconnect to the same session returns the same `participant_id`.
    Joined {
        session_id: String,
        participant_id: String,
    },
    Left {
        session_id: String,
        participant_id: String,
    },
    /// The caller is authorized but not yet admitted. No participant exists
    /// until a capable principal approves the request.
    JoinWaiting {
        session_id: String,
        request_id: String,
    },
    ParticipantJoined {
        session_id: String,
        participant_id: String,
    },
    ParticipantLeft {
        session_id: String,
        participant_id: String,
    },
    /// A complete ordered snapshot, including an empty list for silence.
    ActiveSpeakers {
        session_id: String,
        participant_ids: Vec<String>,
    },
    /// `code` is stable and machine-readable; `message` carries no internal
    /// detail.
    JoinRejected {
        code: String,
        message: String,
    },
    /// A participant's mute state changed. Fanned out to the session so every
    /// client's roster reflects it, and sent to the target so it can stop
    /// sending media.
    ParticipantMuted {
        session_id: String,
        participant_id: String,
        actor_id: String,
        /// "temporary" or "permanent".
        kind: String,
        source: String,
    },
    ParticipantUnmuted {
        session_id: String,
        participant_id: String,
        actor_id: String,
        source: String,
    },
    /// A participation ended. `reason` is "removed" today; carried explicitly so
    /// a later reason does not need a new frame.
    ParticipantRemoved {
        session_id: String,
        participant_id: String,
        actor_id: String,
        reason: String,
    },
    PrincipalBlocked {
        #[serde(skip_serializing_if = "Option::is_none")]
        room_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        actor_id: String,
    },
    PrincipalUnblocked {
        #[serde(skip_serializing_if = "Option::is_none")]
        room_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        actor_id: String,
    },
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
