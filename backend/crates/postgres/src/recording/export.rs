use crate::{PostgresRecordingStore, insert_job_in_tx, stores::sqlx_store_error};
use mahfel_application::{
    jobs::JobEnvelope,
    ports::{BoxFuture, ExportArtifact, ExportSource, RecordingExportStore, StoreError},
};
use mahfel_domain::{ActorId, RecordingExport, RecordingExportStatus, SessionId};
use sqlx::{FromRow, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(FromRow)]
struct ExportRow {
    id: Uuid,
    recording_id: Uuid,
    status: String,
    progress: i32,
    duration_ms: Option<i64>,
    size_bytes: Option<i64>,
    failure_code: Option<String>,
    object_key: Option<String>,
    partial: bool,
}
impl ExportRow {
    fn into_export(self) -> Result<RecordingExport, StoreError> {
        let status = match self.status.as_str() {
            "queued" => RecordingExportStatus::Queued,
            "rendering" => RecordingExportStatus::Rendering,
            "ready" => RecordingExportStatus::Ready,
            "failed" => RecordingExportStatus::Failed,
            _ => return Err(StoreError::Unexpected("invalid export status".into())),
        };
        Ok(RecordingExport {
            id: self.id,
            recording_id: self.recording_id,
            status,
            progress: u16::try_from(self.progress)
                .map_err(|_| StoreError::Unexpected("invalid export progress".into()))?,
            duration_ms: self.duration_ms,
            size_bytes: self.size_bytes,
            failure_code: self.failure_code,
            partial: self.partial,
            object_key: self.object_key,
        })
    }
}

pub(super) async fn enqueue_export(
    tx: &mut Transaction<'_, Postgres>,
    recording_id: Uuid,
    manifest_key: &str,
) -> Result<(), sqlx::Error> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM mahfel.recording_exports WHERE recording_id = $1 AND profile_version = 1)").bind(recording_id).fetch_one(&mut **tx).await?;
    if exists {
        return Ok(());
    }
    let source = ExportSource {
        export_id: Uuid::new_v4(),
        recording_id,
        manifest_key: manifest_key.into(),
        revision: 1,
    };
    let job = JobEnvelope::recording_export(&source, OffsetDateTime::now_utc());
    insert_job_in_tx(tx, &job).await?;
    sqlx::query("INSERT INTO mahfel.recording_exports (id, recording_id, manifest_key, job_id) VALUES ($1,$2,$3,$4)").bind(source.export_id).bind(recording_id).bind(manifest_key).bind(job.job_id).execute(&mut **tx).await?;
    Ok(())
}

impl RecordingExportStore for PostgresRecordingStore {
    fn can_download(
        &self,
        session: SessionId,
        actor: ActorId,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM mahfel.sessions s JOIN mahfel.rooms r ON r.id = s.room_id JOIN mahfel.room_memberships m ON m.room_id = r.id WHERE s.id = $1 AND m.actor_id = $2 AND r.status = 'active' AND NOT EXISTS (SELECT 1 FROM mahfel.blocks b WHERE b.actor_id = $2 AND (b.room_id = r.id OR b.session_id = s.id) AND b.lifted_at IS NULL AND (b.expires_at IS NULL OR b.expires_at > clock_timestamp())))").bind(session.as_uuid()).bind(actor.as_uuid()).fetch_one(&self.pool).await.map_err(sqlx_store_error)
        })
    }
    fn get(
        &self,
        session: SessionId,
        recording: Uuid,
    ) -> BoxFuture<'_, Result<Option<RecordingExport>, StoreError>> {
        Box::pin(async move {
            sqlx::query_as::<_, ExportRow>("SELECT e.id, e.recording_id, e.status, e.progress, e.duration_ms, e.size_bytes, e.failure_code, e.object_key, r.partial_coverage AS partial FROM mahfel.recording_exports e JOIN mahfel.recordings r ON r.id = e.recording_id WHERE r.session_id = $1 AND r.id = $2 AND r.deleted_at IS NULL AND e.manifest_key = r.manifest_key AND e.profile_version = 1").bind(session.as_uuid()).bind(recording).fetch_optional(&self.pool).await.map_err(sqlx_store_error)?.map(ExportRow::into_export).transpose()
        })
    }
    fn retry(
        &self,
        session: SessionId,
        recording: Uuid,
    ) -> BoxFuture<'_, Result<RecordingExport, StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let key: String = sqlx::query_scalar("SELECT manifest_key FROM mahfel.recordings WHERE id = $1 AND session_id = $2 AND status = 'ready' AND deleted_at IS NULL FOR UPDATE").bind(recording).bind(session.as_uuid()).fetch_optional(&mut *tx).await.map_err(sqlx_store_error)?.ok_or_else(|| StoreError::Conflict("recording not ready".into()))?;
            let row: Option<(Uuid, i32, String)> = sqlx::query_as("SELECT id, revision, status FROM mahfel.recording_exports WHERE recording_id = $1 AND manifest_key = $2 AND profile_version = 1 FOR UPDATE").bind(recording).bind(&key).fetch_optional(&mut *tx).await.map_err(sqlx_store_error)?;
            if let Some((id, revision, status)) = row {
                if status == "failed" {
                    let source = ExportSource {
                        export_id: id,
                        recording_id: recording,
                        manifest_key: key,
                        revision: revision.checked_add(1).ok_or_else(|| {
                            StoreError::Conflict("export revision exhausted".into())
                        })?,
                    };
                    let job = JobEnvelope::recording_export(&source, OffsetDateTime::now_utc());
                    insert_job_in_tx(&mut tx, &job)
                        .await
                        .map_err(sqlx_store_error)?;
                    sqlx::query("UPDATE mahfel.recording_exports SET revision = $2, job_id = $3, status = 'queued', progress = 0, failure_code = NULL, updated_at = clock_timestamp() WHERE id = $1").bind(id).bind(source.revision).bind(job.job_id).execute(&mut *tx).await.map_err(sqlx_store_error)?;
                }
            } else {
                enqueue_export(&mut tx, recording, &key)
                    .await
                    .map_err(sqlx_store_error)?;
            }
            tx.commit().await.map_err(sqlx_store_error)?;
            self.get(session, recording)
                .await?
                .ok_or_else(|| StoreError::NotFound("export".into()))
        })
    }
    fn begin<'a>(
        &'a self,
        job: &'a JobEnvelope,
        token: Uuid,
    ) -> BoxFuture<'a, Result<Option<ExportSource>, StoreError>> {
        Box::pin(async move {
            let source: ExportSource = serde_json::from_value(job.payload.clone())
                .map_err(|_| StoreError::Unexpected("invalid export payload".into()))?;
            let updated = sqlx::query("UPDATE mahfel.recording_exports e SET status = 'rendering', failure_code = NULL, updated_at = clock_timestamp() FROM mahfel.background_jobs j, mahfel.recordings r WHERE e.id = $1 AND e.job_id = j.id AND j.id = $2 AND j.claim_token = $3 AND j.status = 'running' AND j.lease_until > clock_timestamp() AND j.generation = $4 AND j.attempt = $5 AND e.revision = $6 AND e.status <> 'ready' AND r.id = e.recording_id AND r.status = 'ready' AND r.manifest_key = e.manifest_key AND e.manifest_key = $7").bind(source.export_id).bind(job.job_id).bind(token).bind(i32::try_from(job.generation).unwrap_or(0)).bind(i16::try_from(job.attempt).unwrap_or(0)).bind(source.revision).bind(&source.manifest_key).execute(&self.pool).await.map_err(sqlx_store_error)?;
            if updated.rows_affected() == 1 {
                let key = format!(
                    "recordings/{}/exports/{}/{}-{token}.mp4",
                    source.recording_id, source.export_id, source.revision
                );
                sqlx::query("INSERT INTO mahfel.recording_export_attempts (claim_token, export_id, object_key) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING").bind(token).bind(source.export_id).bind(key).execute(&self.pool).await.map_err(sqlx_store_error)?;
            }
            Ok((updated.rows_affected() == 1).then_some(source))
        })
    }
    fn accept<'a>(
        &'a self,
        job: &'a JobEnvelope,
        token: Uuid,
        artifact: &'a ExportArtifact,
    ) -> BoxFuture<'a, Result<bool, StoreError>> {
        Box::pin(async move {
            let source: ExportSource = serde_json::from_value(job.payload.clone())
                .map_err(|_| StoreError::Unexpected("invalid export payload".into()))?;
            let updated = sqlx::query("UPDATE mahfel.recording_exports e SET status = 'ready', progress = 100, object_key = $4, duration_ms = $5, size_bytes = $6, sha_256 = $7, failure_code = NULL, updated_at = clock_timestamp() FROM mahfel.background_jobs j, mahfel.recordings r WHERE e.id = $1 AND e.job_id = j.id AND j.id = $2 AND j.claim_token = $3 AND j.status = 'running' AND j.lease_until > clock_timestamp() AND j.generation = $8 AND j.attempt = $9 AND e.revision = $10 AND r.id = e.recording_id AND r.status = 'ready' AND r.manifest_key = e.manifest_key AND e.manifest_key = $11 AND EXISTS (SELECT 1 FROM mahfel.recording_export_attempts a WHERE a.claim_token = $3 AND a.object_key = $4 AND NOT a.retired)").bind(source.export_id).bind(job.job_id).bind(token).bind(&artifact.object_key).bind(artifact.duration_ms).bind(artifact.size_bytes).bind(&artifact.sha256).bind(i32::try_from(job.generation).unwrap_or(0)).bind(i16::try_from(job.attempt).unwrap_or(0)).bind(source.revision).bind(&source.manifest_key).execute(&self.pool).await.map_err(sqlx_store_error)?;
            Ok(updated.rows_affected() == 1)
        })
    }
}

impl PostgresRecordingStore {
    pub async fn export_progress(
        &self,
        id: Uuid,
        token: Uuid,
        progress: i32,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE mahfel.recording_exports e SET progress = GREATEST(e.progress, $3), updated_at = clock_timestamp() FROM mahfel.background_jobs j WHERE e.id = $1 AND j.id = e.job_id AND j.claim_token = $2 AND j.status = 'running' AND j.lease_until > clock_timestamp() AND e.status = 'rendering' AND EXISTS (SELECT 1 FROM mahfel.recordings r WHERE r.id = e.recording_id AND r.deleted_at IS NULL)").bind(id).bind(token).bind(progress.clamp(0,99)).execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }
}

impl PostgresRecordingStore {
    pub async fn export_claim_active(&self, token: Uuid) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM mahfel.background_jobs WHERE claim_token = $1 AND status = 'running' AND lease_until > clock_timestamp())").bind(token).fetch_one(&self.pool).await
    }
    pub async fn export_orphans(&self) -> Result<Vec<(Uuid, String)>, sqlx::Error> {
        sqlx::query_as("UPDATE mahfel.recording_export_attempts a SET retired = TRUE WHERE a.created_at < clock_timestamp() - INTERVAL '1 day' AND a.claim_token IN (SELECT pending.claim_token FROM mahfel.recording_export_attempts pending JOIN mahfel.recording_exports e ON e.id = pending.export_id JOIN mahfel.background_jobs j ON j.id = e.job_id WHERE pending.created_at < clock_timestamp() - INTERVAL '1 day' AND NOT EXISTS (SELECT 1 FROM mahfel.recording_exports accepted WHERE accepted.object_key = pending.object_key) AND NOT (j.status = 'running' AND j.claim_token = pending.claim_token AND j.lease_until > clock_timestamp()) ORDER BY pending.created_at LIMIT 100) RETURNING claim_token, object_key").fetch_all(&self.pool).await
    }
    pub async fn finish_export_cleanup(&self, token: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM mahfel.recording_export_attempts WHERE claim_token = $1 AND retired",
        )
        .bind(token)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
