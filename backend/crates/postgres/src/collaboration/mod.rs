mod access;
mod assets;
mod checkpoints;
mod events;
mod presentation;
mod rows;

use mahfel_application::ports::{
    BoxFuture, CollaborationAccess, CollaborationCheckpoint, CollaborationParticipant,
    CollaborationStore, NewCollaborationEvent, PresentationState, StoreError,
};
use mahfel_domain::{
    ActorId, CollaborationDocument, CollaborationDocumentId, CollaborationEvent, CollaborationKind,
    FileId, SessionId,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::stores::sqlx_store_error;

#[derive(Clone)]
pub struct PostgresCollaborationStore {
    pool: PgPool,
}

impl PostgresCollaborationStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl CollaborationStore for PostgresCollaborationStore {
    fn document(
        &self,
        session_id: SessionId,
        kind: CollaborationKind,
    ) -> BoxFuture<'_, Result<Option<CollaborationDocument>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, rows::DocumentRow>(
                "INSERT INTO mahfel.collaboration_documents (id, session_id, kind, schema_version) \
                 SELECT $1, id, $3, $4 FROM mahfel.sessions WHERE id = $2 \
                 ON CONFLICT (session_id, kind) DO UPDATE SET kind = EXCLUDED.kind \
                 RETURNING id, session_id, kind, schema_version",
            )
            .bind(Uuid::new_v4())
            .bind(session_id.as_uuid())
            .bind(kind.as_str())
            .bind(CollaborationKind::SCHEMA_VERSION)
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            row.map(rows::DocumentRow::into_document).transpose()
        })
    }

    fn access(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<CollaborationAccess, StoreError>> {
        Box::pin(access::access(&self.pool, session_id, actor_id))
    }

    fn register_image(
        &self,
        session_id: SessionId,
        kind: CollaborationKind,
        actor_id: ActorId,
        file_id: FileId,
    ) -> BoxFuture<'_, Result<(), StoreError>> {
        Box::pin(assets::register_image(
            &self.pool, session_id, kind, actor_id, file_id,
        ))
    }

    fn set_edit_grant(
        &self,
        session_id: SessionId,
        manager: ActorId,
        target: ActorId,
        can_edit: bool,
        event_id: Uuid,
    ) -> BoxFuture<'_, Result<CollaborationEvent, StoreError>> {
        Box::pin(access::set_edit_grant(
            &self.pool, session_id, manager, target, can_edit, event_id,
        ))
    }

    fn append<'a>(
        &'a self,
        event: &'a NewCollaborationEvent,
    ) -> BoxFuture<'a, Result<CollaborationEvent, StoreError>> {
        Box::pin(events::append(&self.pool, event))
    }

    fn participants(
        &self,
        session_id: SessionId,
        manager: ActorId,
    ) -> BoxFuture<'_, Result<Vec<CollaborationParticipant>, StoreError>> {
        Box::pin(access::participants(&self.pool, session_id, manager))
    }

    fn presentation(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<PresentationState>, StoreError>> {
        Box::pin(presentation::current(&self.pool, session_id, actor_id))
    }

    fn events(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
        after_sequence: i64,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<CollaborationEvent>, StoreError>> {
        Box::pin(events::list(
            &self.pool,
            session_id,
            actor_id,
            after_sequence,
            limit,
        ))
    }

    fn latest_viewports(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Vec<CollaborationEvent>, StoreError>> {
        Box::pin(events::latest_viewports(&self.pool, session_id, actor_id))
    }

    fn checkpoint(
        &self,
        document_id: CollaborationDocumentId,
    ) -> BoxFuture<'_, Result<Option<CollaborationCheckpoint>, StoreError>> {
        Box::pin(checkpoints::latest(&self.pool, document_id))
    }

    fn save_checkpoint<'a>(
        &'a self,
        document_id: CollaborationDocumentId,
        through_sequence: i64,
        update: &'a [u8],
        state_vector: &'a [u8],
        sha256: &'a [u8],
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(checkpoints::save(
            &self.pool,
            document_id,
            through_sequence,
            update,
            state_vector,
            sha256,
        ))
    }
}
