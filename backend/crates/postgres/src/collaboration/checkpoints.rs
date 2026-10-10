use mahfel_application::ports::{CollaborationCheckpoint, StoreError};
use mahfel_domain::CollaborationDocumentId;
use sqlx::PgPool;

use crate::stores::sqlx_store_error;

pub(super) async fn latest(
    pool: &PgPool,
    document_id: CollaborationDocumentId,
) -> Result<Option<CollaborationCheckpoint>, StoreError> {
    let row = sqlx::query_as::<_, (i64, Vec<u8>, Vec<u8>, Vec<u8>)>(
        "SELECT through_sequence, update_bytes, state_vector, sha_256 \
         FROM mahfel.collaboration_checkpoints WHERE document_id = $1 \
         ORDER BY through_sequence DESC LIMIT 1",
    )
    .bind(document_id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(sqlx_store_error)?;
    Ok(row.map(
        |(through_sequence, update, state_vector, sha256)| CollaborationCheckpoint {
            through_sequence,
            update,
            state_vector,
            sha256,
        },
    ))
}

pub(super) async fn save(
    pool: &PgPool,
    document_id: CollaborationDocumentId,
    through_sequence: i64,
    update: &[u8],
    state_vector: &[u8],
    sha256: &[u8],
) -> Result<(), StoreError> {
    let mut tx = pool.begin().await.map_err(sqlx_store_error)?;
    sqlx::query("SELECT id FROM mahfel.collaboration_documents WHERE id = $1 FOR UPDATE")
        .bind(document_id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(sqlx_store_error)?;
    sqlx::query(
        "INSERT INTO mahfel.collaboration_checkpoints \
         (document_id, through_sequence, update_bytes, state_vector, sha_256) \
         VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
    )
    .bind(document_id.as_uuid())
    .bind(through_sequence)
    .bind(update)
    .bind(state_vector)
    .bind(sha256)
    .execute(&mut *tx)
    .await
    .map_err(sqlx_store_error)?;
    sqlx::query(
        "DELETE FROM mahfel.collaboration_checkpoints WHERE document_id = $1 \
         AND through_sequence NOT IN (SELECT through_sequence FROM mahfel.collaboration_checkpoints \
         WHERE document_id = $1 ORDER BY through_sequence DESC LIMIT 3)",
    )
    .bind(document_id.as_uuid()).execute(&mut *tx).await.map_err(sqlx_store_error)?;
    tx.commit().await.map_err(sqlx_store_error)?;
    Ok(())
}
