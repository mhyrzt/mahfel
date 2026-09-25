use mahfel_application::ports::StoreError;
use mahfel_domain::{ActorId, CollaborationKind, FileId, SessionId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::stores::sqlx_store_error;

use super::access;

pub(super) async fn register_image(
    pool: &PgPool,
    session_id: SessionId,
    kind: CollaborationKind,
    actor_id: ActorId,
    file_id: FileId,
) -> Result<(), StoreError> {
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    access::lock_session(&mut tx, session_id).await?;
    let permission = access::access_in_tx(&mut tx, session_id, actor_id).await?;
    if !permission.can_edit || !permission.live {
        return Err(StoreError::NotFound("collaboration editing".into()));
    }
    let document_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM mahfel.collaboration_documents \
         WHERE session_id = $1 AND kind = $2 AND schema_version = $3",
    )
    .bind(session_id.as_uuid())
    .bind(kind.as_str())
    .bind(CollaborationKind::SCHEMA_VERSION)
    .fetch_optional(&mut *tx)
    .await
    .map_err(sqlx_store_error)?
    .ok_or_else(|| StoreError::NotFound("collaboration document".into()))?;
    let mime: String = sqlx::query_scalar(
        "SELECT verified_mime FROM mahfel.files \
         WHERE id = $1 AND session_id = $2 AND status = 'ready' FOR UPDATE",
    )
    .bind(file_id.as_uuid())
    .bind(session_id.as_uuid())
    .fetch_optional(&mut *tx)
    .await
    .map_err(sqlx_store_error)?
    .ok_or_else(|| StoreError::NotFound("session image".into()))?;
    if !supported_image_mime(&mime) {
        return Err(StoreError::Conflict(
            "unsupported collaboration image".into(),
        ));
    }
    sqlx::query(
        "INSERT INTO mahfel.collaboration_document_assets (document_id, file_id) \
         VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(document_id)
    .bind(file_id.as_uuid())
    .execute(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(())
}

fn supported_image_mime(mime: &str) -> bool {
    matches!(mime, "image/png" | "image/jpeg")
}

#[cfg(test)]
mod tests {
    use super::supported_image_mime;

    #[test]
    fn collaboration_assets_accept_only_verified_raster_images() {
        assert!(supported_image_mime("image/png"));
        assert!(supported_image_mime("image/jpeg"));
        assert!(!supported_image_mime("application/pdf"));
        assert!(!supported_image_mime("image/svg+xml"));
    }
}
