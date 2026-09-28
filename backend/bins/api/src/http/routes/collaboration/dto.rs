use mahfel_domain::{CollaborationDocument, CollaborationEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct DocumentResponse {
    pub id: String,
    pub kind: String,
    pub schema_version: i16,
}

impl From<CollaborationDocument> for DocumentResponse {
    fn from(value: CollaborationDocument) -> Self {
        Self {
            id: value.id.to_string(),
            kind: value.kind.as_str().to_owned(),
            schema_version: value.schema_version,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub(crate) struct CollaborationDocumentsResponse {
    pub room_id: String,
    pub documents: Vec<DocumentResponse>,
    pub can_edit: bool,
    pub can_publish_audio: bool,
    pub can_publish_video: bool,
    pub can_share_screen: bool,
    pub can_present: bool,
    pub can_manage: bool,
    pub live: bool,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ParticipantGrantResponse {
    pub actor_id: String,
    pub display_name: String,
    pub can_edit: bool,
    pub can_publish_audio: bool,
    pub can_publish_video: bool,
    pub can_share_screen: bool,
    pub can_present: bool,
    pub is_manager: bool,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ParticipantGrantsResponse {
    pub participants: Vec<ParticipantGrantResponse>,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct ParticipantAccessRequest {
    pub can_edit: Option<bool>,
    pub can_publish_audio: Option<bool>,
    pub can_publish_video: Option<bool>,
    pub can_share_screen: Option<bool>,
    pub can_present: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct RegisterImageRequest {
    pub file_id: uuid::Uuid,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct ViewportRequest {
    pub surface: String,
    pub x: f64,
    pub y: f64,
    pub zoom: Option<f64>,
}

#[derive(Deserialize, ToSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub(crate) enum PresentationRequest {
    Select { file_ids: Vec<uuid::Uuid> },
    Page { page: u32 },
    Stop,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct EventResponse {
    pub sequence: i64,
    pub id: String,
    pub actor_id: String,
    pub document_id: Option<String>,
    pub kind: String,
    pub update_base64: Option<String>,
    pub data: Value,
    pub created_at: String,
}

impl From<CollaborationEvent> for EventResponse {
    fn from(value: CollaborationEvent) -> Self {
        use base64::Engine;
        Self {
            sequence: value.sequence,
            id: value.id.to_string(),
            actor_id: value.actor_id.to_string(),
            document_id: value.document_id.map(|id| id.to_string()),
            kind: value.kind.as_str().to_owned(),
            update_base64: value
                .update
                .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes)),
            data: value.data,
            created_at: value.created_at.to_string(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub(crate) struct CollaborationEventsResponse {
    pub events: Vec<EventResponse>,
}

#[derive(Deserialize)]
pub(crate) struct EventsQuery {
    pub after_sequence: Option<i64>,
    pub limit: Option<u32>,
}
