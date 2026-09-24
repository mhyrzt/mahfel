use mahfel_application::ports::StoreError;
use mahfel_domain::{
    ActorId, CollaborationDocument, CollaborationDocumentId, CollaborationEvent,
    CollaborationEventKind, CollaborationKind, SessionId,
};
use serde_json::Value;
use sqlx::types::Json;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub(super) struct DocumentRow {
    id: Uuid,
    session_id: Uuid,
    kind: String,
    schema_version: i16,
}

impl DocumentRow {
    pub(super) fn into_document(self) -> Result<CollaborationDocument, StoreError> {
        Ok(CollaborationDocument {
            id: CollaborationDocumentId::from_uuid(self.id),
            session_id: SessionId::from_uuid(self.session_id),
            kind: CollaborationKind::from_db(&self.kind)
                .ok_or_else(|| StoreError::Unexpected("invalid document kind".into()))?,
            schema_version: self.schema_version,
        })
    }
}

#[derive(sqlx::FromRow)]
pub(super) struct EventRow {
    pub(super) session_id: Uuid,
    sequence: i64,
    id: Uuid,
    pub(super) actor_id: Uuid,
    document_id: Option<Uuid>,
    kind: String,
    update_bytes: Option<Vec<u8>>,
    data: Json<Value>,
    created_at: OffsetDateTime,
}

impl EventRow {
    pub(super) fn into_event(self) -> Result<CollaborationEvent, StoreError> {
        Ok(CollaborationEvent {
            session_id: SessionId::from_uuid(self.session_id),
            sequence: self.sequence,
            id: self.id,
            actor_id: ActorId::from_uuid(self.actor_id),
            document_id: self.document_id.map(CollaborationDocumentId::from_uuid),
            kind: CollaborationEventKind::from_db(&self.kind)
                .ok_or_else(|| StoreError::Unexpected("invalid event kind".into()))?,
            update: self.update_bytes,
            data: self.data.0,
            created_at: self.created_at,
        })
    }
}
