use mahfel_application::ports::{NewCollaborationEvent, StoreError};
use mahfel_domain::{
    ActorId, CollaborationDocumentId, CollaborationEvent, CollaborationEventKind, SessionId,
};
use serde_json::Value;
use sqlx::{PgConnection, PgPool, Postgres, Transaction, types::Json};
use uuid::Uuid;

use crate::stores::sqlx_store_error;

use super::{access, presentation, rows::EventRow};

pub(super) async fn append(
    pool: &PgPool,
    event: &NewCollaborationEvent,
) -> Result<CollaborationEvent, StoreError> {
    if event.kind == CollaborationEventKind::EditGrant {
        return Err(StoreError::Unexpected(
            "grant requires manager operation".into(),
        ));
    }
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    access::lock_session(&mut tx, event.session_id).await?;
    let allowed = access::access_in_tx(&mut tx, event.session_id, event.actor_id).await?;
    if !allowed.can_read || !allowed.live {
        return Err(StoreError::NotFound("live collaboration".into()));
    }
    if event.kind != CollaborationEventKind::Viewport && !allowed.can_edit {
        return Err(StoreError::NotFound("collaboration editing".into()));
    }
    match event.kind {
        CollaborationEventKind::DocumentUpdate => {
            let Some(document_id) = event.document_id else {
                return Err(StoreError::Unexpected(
                    "document update missing document".into(),
                ));
            };
            if event.update.as_ref().is_none_or(Vec::is_empty) {
                return Err(StoreError::Unexpected("empty document update".into()));
            }
            let owner = sqlx::query_scalar::<_, Uuid>(
                "SELECT session_id FROM mahfel.collaboration_documents WHERE id = $1",
            )
            .bind(document_id.as_uuid())
            .fetch_optional(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            if owner != Some(event.session_id.as_uuid()) {
                return Err(StoreError::NotFound("collaboration document".into()));
            }
        }
        _ if event.document_id.is_some() || event.update.is_some() => {
            return Err(StoreError::Unexpected(
                "invalid collaboration event shape".into(),
            ));
        }
        CollaborationEventKind::Presentation => {
            presentation::validate(&mut tx, event.session_id, &event.data).await?;
        }
        _ => {}
    }
    let inserted = insert_in_tx(
        &mut tx,
        event.session_id,
        event.id,
        event.actor_id,
        event.document_id,
        event.kind,
        event.update.as_deref(),
        &event.data,
    )
    .await?;
    if event.kind == CollaborationEventKind::Presentation {
        presentation::apply(&mut tx, event.session_id, inserted.id, &event.data).await?;
    }
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(inserted)
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn insert_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    session_id: SessionId,
    id: Uuid,
    actor_id: ActorId,
    document_id: Option<CollaborationDocumentId>,
    kind: CollaborationEventKind,
    update: Option<&[u8]>,
    data: &Value,
) -> Result<CollaborationEvent, StoreError> {
    let existing = sqlx::query_as::<_, EventRow>(
        "SELECT session_id, event_sequence AS sequence, id, actor_id, document_id, kind, update_bytes, payload AS data, created_at \
         FROM mahfel.collaboration_events WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(sqlx_store_error)?;
    if let Some(existing) = existing {
        if existing.session_id != session_id.as_uuid() || existing.actor_id != actor_id.as_uuid() {
            return Err(StoreError::Conflict("event id reused".into()));
        }
        return existing.into_event();
    }
    let sequence = next_sequence(tx, session_id).await?;
    sqlx::query_as::<_, EventRow>(
        "INSERT INTO mahfel.collaboration_events \
         (session_id, event_sequence, id, actor_id, document_id, kind, update_bytes, payload) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8) \
         RETURNING session_id, event_sequence AS sequence, id, actor_id, document_id, kind, update_bytes, payload AS data, created_at",
    )
    .bind(session_id.as_uuid())
    .bind(sequence)
    .bind(id)
    .bind(actor_id.as_uuid())
    .bind(document_id.map(CollaborationDocumentId::as_uuid))
    .bind(kind.as_str())
    .bind(update)
    .bind(Json(data))
    .fetch_one(&mut **tx)
    .await
    .map_err(sqlx_store_error)?
    .into_event()
}

async fn next_sequence(
    connection: &mut PgConnection,
    session_id: SessionId,
) -> Result<i64, StoreError> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO mahfel.collaboration_sequences (session_id, next_sequence) VALUES ($1, 2) \
         ON CONFLICT (session_id) DO UPDATE \
         SET next_sequence = mahfel.collaboration_sequences.next_sequence + 1 \
         RETURNING next_sequence - 1",
    )
    .bind(session_id.as_uuid())
    .fetch_one(connection)
    .await
    .map_err(sqlx_store_error)
}

pub(super) async fn list(
    pool: &PgPool,
    session_id: SessionId,
    actor_id: ActorId,
    after_sequence: i64,
    limit: u32,
) -> Result<Vec<CollaborationEvent>, StoreError> {
    if after_sequence < 0 || limit == 0 || limit > 100 {
        return Err(StoreError::Unexpected(
            "invalid collaboration cursor".into(),
        ));
    }
    if !access::access(pool, session_id, actor_id).await?.can_read {
        return Err(StoreError::NotFound("collaboration session".into()));
    }
    sqlx::query_as::<_, EventRow>(
        "SELECT session_id, event_sequence AS sequence, id, actor_id, document_id, kind, update_bytes, payload AS data, created_at \
         FROM mahfel.collaboration_events \
         WHERE session_id = $1 AND event_sequence > $2 ORDER BY event_sequence LIMIT $3",
    )
    .bind(session_id.as_uuid())
    .bind(after_sequence)
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await
    .map_err(sqlx_store_error)?
    .into_iter()
    .map(EventRow::into_event)
    .collect()
}

pub(super) async fn latest_viewports(
    pool: &PgPool,
    session_id: SessionId,
    actor_id: ActorId,
) -> Result<Vec<CollaborationEvent>, StoreError> {
    if !access::access(pool, session_id, actor_id).await?.can_read {
        return Err(StoreError::NotFound("collaboration session".into()));
    }
    sqlx::query_as::<_, EventRow>(
        "SELECT DISTINCT ON (payload ->> 'surface') session_id, event_sequence AS sequence, \
         id, actor_id, document_id, kind, update_bytes, payload AS data, created_at \
         FROM mahfel.collaboration_events \
         WHERE session_id = $1 AND kind = 'viewport' \
         ORDER BY payload ->> 'surface', event_sequence DESC",
    )
    .bind(session_id.as_uuid())
    .fetch_all(pool)
    .await
    .map_err(sqlx_store_error)?
    .into_iter()
    .map(EventRow::into_event)
    .collect()
}
