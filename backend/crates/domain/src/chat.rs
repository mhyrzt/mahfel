use crate::actor::ActorId;
use crate::ids::entity_id;
use crate::room::RoomId;
use crate::session::SessionId;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

pub const CHAT_AUTHOR_MUTATION_WINDOW: time::Duration = time::Duration::minutes(15);

entity_id!(ChatChannelId);
entity_id!(ChatMessageId);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatChannel {
    pub id: ChatChannelId,
    pub room_id: RoomId,
    pub created_at: OffsetDateTime,
}

impl ChatChannel {
    #[must_use]
    pub fn for_room(id: ChatChannelId, room_id: RoomId, created_at: OffsetDateTime) -> Self {
        Self {
            id,
            room_id,
            created_at,
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ChatMessageError {
    #[error("message content cannot be empty")]
    EmptyContent,
    #[error("message exceeds 4000 characters")]
    ContentTooLong,
    #[error("message has too many mentions")]
    TooManyMentions,
    #[error("message mentions an actor more than once")]
    DuplicateMention,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatMessageDraft {
    pub id: ChatMessageId,
    pub author_id: ActorId,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub parent_id: Option<ChatMessageId>,
    pub mentions: Vec<ActorId>,
    /// When set, the message belongs to a live session conversation instead of
    /// the general room conversation.
    pub session_id: Option<SessionId>,
}

impl ChatMessageDraft {
    pub fn new(
        id: ChatMessageId,
        author_id: ActorId,
        content: String,
        created_at: OffsetDateTime,
    ) -> Result<Self, ChatMessageError> {
        if content.trim().is_empty() {
            return Err(ChatMessageError::EmptyContent);
        }
        if content.chars().count() > 4000 {
            return Err(ChatMessageError::ContentTooLong);
        }

        Ok(Self {
            id,
            author_id,
            content,
            created_at,
            parent_id: None,
            mentions: Vec::new(),
            session_id: None,
        })
    }

    #[must_use]
    pub fn with_session(mut self, session_id: Option<SessionId>) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn with_details(
        mut self,
        parent_id: Option<ChatMessageId>,
        mentions: Vec<ActorId>,
    ) -> Result<Self, ChatMessageError> {
        if mentions.len() > 20 {
            return Err(ChatMessageError::TooManyMentions);
        }
        let unique = mentions
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        if unique.len() != mentions.len() {
            return Err(ChatMessageError::DuplicateMention);
        }
        self.parent_id = parent_id;
        self.mentions = mentions;
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: ChatMessageId,
    pub channel_id: ChatChannelId,
    pub author_id: ActorId,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub edited_at: Option<OffsetDateTime>,
    /// Set by soft deletion; the row remains as a tombstone for stable history.
    pub deleted_at: Option<OffsetDateTime>,
    pub parent_id: Option<ChatMessageId>,
    pub session_id: Option<SessionId>,
}

impl ChatMessage {
    #[must_use]
    pub fn from_draft(channel_id: ChatChannelId, draft: ChatMessageDraft) -> Self {
        Self {
            id: draft.id,
            channel_id,
            author_id: draft.author_id,
            content: draft.content,
            created_at: draft.created_at,
            edited_at: None,
            deleted_at: None,
            parent_id: draft.parent_id,
            session_id: draft.session_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatMessageCursor {
    pub created_at: OffsetDateTime,
    pub message_id: ChatMessageId,
}

impl ChatMessageCursor {
    #[must_use]
    pub const fn new(created_at: OffsetDateTime, message_id: ChatMessageId) -> Self {
        Self {
            created_at,
            message_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatMessagePage {
    pub messages: Vec<ChatMessage>,
    pub next_cursor: Option<ChatMessageCursor>,
}

/// A committed change to one room channel. Sequence numbers are gapless within
/// a channel and can be used to resume a WebSocket subscription.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatEvent {
    pub id: uuid::Uuid,
    pub sequence: i64,
    pub room_id: RoomId,
    pub kind: ChatEventKind,
    pub message: ChatMessage,
    pub reaction_actor_id: Option<ActorId>,
    pub reaction_emoji: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatEventKind {
    Created,
    Edited,
    Deleted,
    ReactionSet,
    ReactionRemoved,
}

/// One actor may react with exactly one emoji on a message. The supported set
/// is intentionally small and stable for consistent aggregation.
pub const CHAT_REACTION_EMOJIS: &[&str] = &["👍", "👎", "❤️", "😂", "😮", "😢"];
