use crate::{ActorId, SessionId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::CollaborationDocumentId;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationEventKind {
    DocumentUpdate,
    Viewport,
    Presentation,
    EditGrant,
}

impl CollaborationEventKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DocumentUpdate => "document_update",
            Self::Viewport => "viewport",
            Self::Presentation => "presentation",
            Self::EditGrant => "edit_grant",
        }
    }

    #[must_use]
    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "document_update" => Some(Self::DocumentUpdate),
            "viewport" => Some(Self::Viewport),
            "presentation" => Some(Self::Presentation),
            "edit_grant" => Some(Self::EditGrant),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CollaborationEvent {
    pub session_id: SessionId,
    pub sequence: i64,
    pub id: Uuid,
    pub actor_id: ActorId,
    pub document_id: Option<CollaborationDocumentId>,
    pub kind: CollaborationEventKind,
    pub update: Option<Vec<u8>>,
    pub data: serde_json::Value,
    pub created_at: OffsetDateTime,
}
