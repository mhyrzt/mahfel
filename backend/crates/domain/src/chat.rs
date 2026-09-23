use crate::actor::ActorId;
use crate::ids::entity_id;
use crate::room::RoomId;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

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
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatMessageDraft {
    pub id: ChatMessageId,
    pub author_id: ActorId,
    pub content: String,
    pub created_at: OffsetDateTime,
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

        Ok(Self {
            id,
            author_id,
            content,
            created_at,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: ChatMessageId,
    pub channel_id: ChatChannelId,
    pub author_id: ActorId,
    pub content: String,
    pub created_at: OffsetDateTime,
    /// Set by soft deletion; the row remains as a tombstone for stable history.
    pub deleted_at: Option<OffsetDateTime>,
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
            deleted_at: None,
        }
    }
}
