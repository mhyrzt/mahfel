use crate::{PostgresRecordingStore, stores::sqlx_store_error};
use mahfel_application::ports::{
    BoxFuture, RecordingExportStore, RecordingPlaybackStore, StoreError,
};
use mahfel_domain::{ActorId, SessionId};
use uuid::Uuid;

impl RecordingPlaybackStore for PostgresRecordingStore {
    fn retry_processing(
        &self,
        session: SessionId,
        recording: Uuid,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            let job: Option<Uuid> = sqlx::query_scalar(
                "SELECT job.id FROM mahfel.background_jobs job JOIN mahfel.recordings recording ON job.payload ->> 'recording_id' = recording.id::TEXT WHERE recording.session_id = $1 AND recording.id = $2 AND recording.deleted_at IS NULL AND recording.status = 'finalizing' AND job.job_type = 'recordings.finalize' AND job.status = 'dead' ORDER BY job.updated_at DESC LIMIT 1",
            ).bind(session.as_uuid()).bind(recording).fetch_optional(&self.pool).await.map_err(sqlx_store_error)?;
            let Some(job) = job else {
                return Ok(false);
            };
            crate::JobStore::new(self.pool.clone())
                .replay_dead(job)
                .await
                .map_err(sqlx_store_error)
        })
    }
    fn playback_index(
        &self,
        session: SessionId,
        recording: Uuid,
    ) -> BoxFuture<'_, Result<Option<String>, StoreError>> {
        Box::pin(async move {
            sqlx::query_scalar("SELECT playback_index_key FROM mahfel.recordings WHERE session_id = $1 AND id = $2 AND status = 'ready' AND deleted_at IS NULL")
                .bind(session.as_uuid()).bind(recording).fetch_optional(&self.pool).await
                .map(|value: Option<Option<String>>| value.flatten()).map_err(sqlx_store_error)
        })
    }
    fn can_play(
        &self,
        session: SessionId,
        actor: ActorId,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        RecordingExportStore::can_download(self, session, actor)
    }
    fn manifest(
        &self,
        session: SessionId,
        recording: Uuid,
    ) -> BoxFuture<'_, Result<Option<String>, StoreError>> {
        Box::pin(async move {
            sqlx::query_scalar("SELECT manifest_key FROM mahfel.recordings WHERE session_id = $1 AND id = $2 AND status = 'ready' AND playback_index_key IS NOT NULL AND deleted_at IS NULL")
                .bind(session.as_uuid()).bind(recording).fetch_optional(&self.pool).await.map_err(sqlx_store_error)
        })
    }
    fn delete(
        &self,
        session: SessionId,
        recording: Uuid,
        actor: ActorId,
    ) -> BoxFuture<'_, Result<(), StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            sqlx::query("SELECT id FROM mahfel.sessions WHERE id = $1 FOR UPDATE")
                .bind(session.as_uuid())
                .execute(&mut *tx)
                .await
                .map_err(sqlx_store_error)?;
            let row: Option<(String, Option<time::OffsetDateTime>)> = sqlx::query_as("SELECT status, deleted_at FROM mahfel.recordings WHERE id = $1 AND session_id = $2 FOR UPDATE").bind(recording).bind(session.as_uuid()).fetch_optional(&mut *tx).await.map_err(sqlx_store_error)?;
            let (status, deleted) = row.ok_or_else(|| StoreError::NotFound("recording".into()))?;
            if deleted.is_some() {
                return Ok(());
            }
            if !matches!(status.as_str(), "ready" | "failed") {
                return Err(StoreError::Conflict("recording is still processing".into()));
            }
            let revision: i64 = sqlx::query_scalar("UPDATE mahfel.recordings SET deleted_at = clock_timestamp(), status = 'failed', revision = revision + 1 WHERE id = $1 RETURNING revision").bind(recording).fetch_one(&mut *tx).await.map_err(sqlx_store_error)?;
            sqlx::query("INSERT INTO mahfel.recording_audit (recording_id, actor_id, action_kind, reason, revision) VALUES ($1,$2,'delete','manual',$3)").bind(recording).bind(actor.as_uuid()).bind(revision).execute(&mut *tx).await.map_err(sqlx_store_error)?;
            sqlx::query("UPDATE mahfel.recording_assets SET copy_status = 'failed' WHERE recording_id = $1 AND copy_status = 'pending'").bind(recording).execute(&mut *tx).await.map_err(sqlx_store_error)?;
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(())
        })
    }
}

impl PostgresRecordingStore {
    pub async fn deleted_recordings(&self) -> Result<Vec<Uuid>, sqlx::Error> {
        sqlx::query_scalar("SELECT r.id FROM mahfel.recordings r WHERE r.deleted_at IS NOT NULL AND (r.cleanup_checked_at IS NULL OR r.cleanup_checked_at < clock_timestamp() - INTERVAL '1 day') AND NOT EXISTS (SELECT 1 FROM mahfel.background_jobs j WHERE j.payload ->> 'recording_id' = r.id::text AND j.status = 'running' AND j.lease_until > clock_timestamp()) ORDER BY r.cleanup_checked_at NULLS FIRST, r.id LIMIT 20").fetch_all(&self.pool).await
    }
    pub async fn finish_recording_cleanup(&self, recording: Uuid) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        for table in [
            "recording_fragments",
            "recording_events",
            "recording_snapshots",
            "recording_assets",
        ] {
            sqlx::query(sqlx::AssertSqlSafe(format!("DELETE FROM mahfel.{table} WHERE recording_id = $1 AND EXISTS (SELECT 1 FROM mahfel.recordings WHERE id = $1 AND deleted_at IS NOT NULL)"))).bind(recording).execute(&mut *tx).await?;
        }
        sqlx::query("DELETE FROM mahfel.recording_export_attempts WHERE export_id IN (SELECT id FROM mahfel.recording_exports WHERE recording_id = $1)").bind(recording).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM mahfel.recording_exports WHERE recording_id = $1")
            .bind(recording)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE mahfel.recordings SET cleanup_checked_at = clock_timestamp() WHERE id = $1 AND deleted_at IS NOT NULL").bind(recording).execute(&mut *tx).await?;
        tx.commit().await
    }
}
