use crate::ports::{BoxFuture, StoreError};
use mahfel_domain::FileId;
use mahfel_domain::{
    ActorId, CollaborationDocument, CollaborationDocumentId, CollaborationEvent,
    CollaborationEventKind, CollaborationKind, SessionId,
};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CollaborationAccess {
    pub can_read: bool,
    pub can_edit: bool,
    pub can_manage: bool,
    pub live: bool,
}

pub struct NewCollaborationEvent {
    pub session_id: SessionId,
    pub id: Uuid,
    pub actor_id: ActorId,
    pub document_id: Option<CollaborationDocumentId>,
    pub kind: CollaborationEventKind,
    pub update: Option<Vec<u8>>,
    pub data: Value,
}

pub struct CollaborationParticipant {
    pub actor_id: ActorId,
    pub display_name: String,
    pub can_edit: bool,
}

pub struct PresentationState {
    pub file_ids: Vec<FileId>,
    pub current_page: u32,
    pub total_pages: u32,
}

pub struct CollaborationCheckpoint {
    pub through_sequence: i64,
    pub update: Vec<u8>,
    pub state_vector: Vec<u8>,
    pub sha256: Vec<u8>,
}

pub trait CollaborationStore: Send + Sync {
    fn document(
        &self,
        session_id: SessionId,
        kind: CollaborationKind,
    ) -> BoxFuture<'_, Result<Option<CollaborationDocument>, StoreError>>;

    fn access(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<CollaborationAccess, StoreError>>;

    fn set_edit_grant(
        &self,
        session_id: SessionId,
        manager: ActorId,
        target: ActorId,
        can_edit: bool,
        event_id: Uuid,
    ) -> BoxFuture<'_, Result<CollaborationEvent, StoreError>>;

    fn participants(
        &self,
        session_id: SessionId,
        manager: ActorId,
    ) -> BoxFuture<'_, Result<Vec<CollaborationParticipant>, StoreError>>;

    fn presentation(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<PresentationState>, StoreError>>;

    fn append<'a>(
        &'a self,
        event: &'a NewCollaborationEvent,
    ) -> BoxFuture<'a, Result<CollaborationEvent, StoreError>>;

    fn events(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
        after_sequence: i64,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<CollaborationEvent>, StoreError>>;

    fn latest_viewports(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Vec<CollaborationEvent>, StoreError>>;

    fn checkpoint(
        &self,
        document_id: CollaborationDocumentId,
    ) -> BoxFuture<'_, Result<Option<CollaborationCheckpoint>, StoreError>>;

    fn save_checkpoint<'a>(
        &'a self,
        document_id: CollaborationDocumentId,
        through_sequence: i64,
        update: &'a [u8],
        state_vector: &'a [u8],
        sha256: &'a [u8],
    ) -> BoxFuture<'a, Result<(), StoreError>>;
}
