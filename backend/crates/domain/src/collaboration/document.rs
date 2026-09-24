use crate::SessionId;
use crate::ids::entity_id;
use serde::{Deserialize, Serialize};

entity_id!(CollaborationDocumentId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationKind {
    Notes,
    Whiteboard,
}

impl CollaborationKind {
    pub const SCHEMA_VERSION: i16 = 1;

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Notes => "notes",
            Self::Whiteboard => "whiteboard",
        }
    }

    #[must_use]
    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "notes" => Some(Self::Notes),
            "whiteboard" => Some(Self::Whiteboard),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollaborationDocument {
    pub id: CollaborationDocumentId,
    pub session_id: SessionId,
    pub kind: CollaborationKind,
    pub schema_version: i16,
}
