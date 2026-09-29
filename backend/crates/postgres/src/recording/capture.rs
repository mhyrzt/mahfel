use crate::{PostgresRecordingStore, insert_job_in_tx};
use mahfel_application::{jobs::JobEnvelope, recording_artifacts::FragmentReceipt};
use mahfel_domain::SessionId;
use sqlx::{FromRow, types::Json};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub struct CaptureIntent {
    pub recording_id: Uuid,
    pub session_id: Uuid,
    pub room_id: Uuid,
    pub status: String,
    pub revision: i64,
    pub generation: i32,
    pub current_generation: i32,
    pub boot_id: Option<Uuid>,
    pub receipt_prefix: Option<String>,
    pub sealed: bool,
}
#[derive(FromRow)]
pub struct RecordingAssetCopy {
    pub recording_id: Uuid,
    pub file_id: Uuid,
    pub source_key: String,
    pub archive_key: String,
    pub checksum_sha_256: String,
    pub size_bytes: i64,
}
impl PostgresRecordingStore {
    pub async fn capture_intents(
        &self,
        session: Option<SessionId>,
    ) -> Result<Vec<CaptureIntent>, sqlx::Error> {
        sqlx::query_as("SELECT recording.id AS recording_id, recording.session_id, session.room_id, recording.status, recording.revision, generation.generation, recording.capture_generation AS current_generation, generation.boot_id, generation.receipt_prefix, generation.sealed \
            FROM mahfel.recordings recording JOIN mahfel.sessions session ON session.id = recording.session_id \
            JOIN mahfel.recording_generations generation ON generation.recording_id = recording.id \
            WHERE recording.status IN ('starting','recording','paused','stopping') AND (NOT generation.sealed OR generation.generation = recording.capture_generation) \
            AND ($1::UUID IS NULL OR recording.session_id = $1) ORDER BY recording.started_at, generation.generation LIMIT 100")
            .bind(session.map(SessionId::as_uuid)).fetch_all(&self.pool).await
    }
    pub async fn capture_report(
        &self,
        intent: &CaptureIntent,
        observation: &CaptureObservation,
    ) -> Result<bool, sqlx::Error> {
        let CaptureObservation {
            boot,
            origin,
            capturing,
            sealed,
            warning,
        } = observation;
        let boot = *boot;
        let capturing = *capturing;
        let sealed = *sealed;
        let warning = warning.as_deref();
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT id FROM mahfel.sessions WHERE id = $1 FOR UPDATE")
            .bind(intent.session_id)
            .execute(&mut *tx)
            .await?;
        let current: Option<Uuid> = sqlx::query_scalar("SELECT id FROM mahfel.recordings WHERE id = $1 AND revision = $2 AND capture_generation = $3 AND status IN ('starting','recording','paused','stopping') FOR UPDATE")
            .bind(intent.recording_id).bind(intent.revision).bind(intent.current_generation).fetch_optional(&mut *tx).await?;
        if current.is_none() {
            return Ok(false);
        }
        if intent.boot_id.is_some_and(|known| known != boot) {
            sqlx::query("UPDATE mahfel.recording_generations SET sealed = TRUE, ended_at = COALESCE(ended_at, clock_timestamp()), loss_reason = 'media_restart' WHERE recording_id = $1 AND generation = $2")
                .bind(intent.recording_id).bind(intent.generation).execute(&mut *tx).await?;
            sqlx::query("UPDATE mahfel.recordings SET partial_coverage = TRUE, warning = 'media_restart', \
                capture_generation = capture_generation + CASE WHEN status IN ('starting','recording') AND capture_generation = $2 THEN 1 ELSE 0 END, \
                status = CASE WHEN status = 'recording' AND capture_generation = $2 THEN 'starting' ELSE status END, revision = revision + 1 WHERE id = $1")
                .bind(intent.recording_id).bind(intent.generation).execute(&mut *tx).await?;
        } else {
            let prefix = format!(
                "recordings/{}/capture/{}/{boot}/receipts/",
                intent.recording_id, intent.generation
            );
            sqlx::query("UPDATE mahfel.recording_generations SET boot_id = $3, unix_origin_ns = $4::TEXT::NUMERIC, receipt_prefix = $5, sealed = $6, \
                loss_reason = COALESCE($7, loss_reason) WHERE recording_id = $1 AND generation = $2 AND NOT sealed")
                .bind(intent.recording_id).bind(intent.generation).bind(boot).bind(origin.to_string()).bind(prefix).bind(sealed).bind(warning).execute(&mut *tx).await?;
            sqlx::query("UPDATE mahfel.recordings SET status = CASE WHEN status = 'starting' AND $3 THEN 'recording' ELSE status END, \
                warning = $4, partial_coverage = partial_coverage OR ($4::TEXT IS NOT NULL) WHERE id = $1 AND capture_generation = $2")
                .bind(intent.recording_id).bind(intent.generation).bind(capturing).bind(warning).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(true)
    }
    pub async fn capture_unavailable(&self, intent: &CaptureIntent) -> Result<(), sqlx::Error> {
        sqlx::query("WITH affected AS (UPDATE mahfel.recordings SET warning = 'capture_unavailable', partial_coverage = TRUE \
            WHERE id = $1 AND revision = $2 AND capture_generation = $3 AND status IN ('starting','recording','stopping') RETURNING id) \
            UPDATE mahfel.recording_generations SET loss_reason = COALESCE(loss_reason, 'capture_unavailable') \
            WHERE recording_id IN (SELECT id FROM affected) AND generation = $4 AND NOT sealed")
            .bind(intent.recording_id).bind(intent.revision).bind(intent.current_generation).bind(intent.generation).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn import_fragment(&self, receipt: &FragmentReceipt) -> Result<bool, sqlx::Error> {
        let prefix = format!(
            "recordings/{}/capture/{}/{}/tracks/",
            receipt.recording_id, receipt.generation, receipt.boot_id
        );
        if !receipt.object_key.starts_with(&prefix)
            || receipt.bytes == 0
            || receipt.bytes > 32 * 1024 * 1024
            || receipt.end_ns < receipt.start_ns
            || receipt.sha256.len() != 64
            || !receipt.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Ok(false);
        }
        let result = sqlx::query("INSERT INTO mahfel.recording_fragments (recording_id, generation, object_key, receipt) \
            SELECT $1, $2, $3, $4 FROM mahfel.recording_generations generation JOIN mahfel.recordings recording ON recording.id = generation.recording_id \
            WHERE generation.recording_id = $1 AND generation.generation = $2 AND generation.boot_id = $5 AND recording.status IN ('starting','recording','paused','stopping','finalizing') \
            ON CONFLICT DO NOTHING")
            .bind(receipt.recording_id).bind(i32::try_from(receipt.generation).unwrap_or(i32::MAX)).bind(&receipt.object_key).bind(Json(receipt)).bind(receipt.boot_id).execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }
    pub async fn finalize_sealed(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let generation: Option<i32> = sqlx::query_scalar("SELECT capture_generation FROM mahfel.recordings WHERE id = $1 AND status = 'stopping' FOR UPDATE")
            .bind(id).fetch_optional(&mut *tx).await?;
        let Some(generation) = generation else {
            return Ok(false);
        };
        sqlx::query("UPDATE mahfel.recording_generations SET sealed = TRUE, ended_at = COALESCE(ended_at, clock_timestamp()), loss_reason = 'seal_timeout' \
            WHERE recording_id = $1 AND NOT sealed AND EXISTS (SELECT 1 FROM mahfel.recordings WHERE id = $1 AND stopped_at < clock_timestamp() - INTERVAL '30 seconds')")
            .bind(id).execute(&mut *tx).await?;
        let unfinished: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM mahfel.recording_generations WHERE recording_id = $1 AND NOT sealed)")
            .bind(id).fetch_one(&mut *tx).await?;
        if unfinished {
            return Ok(false);
        }
        sqlx::query("UPDATE mahfel.recordings SET status = 'finalizing', warning = CASE WHEN EXISTS (SELECT 1 FROM mahfel.recording_generations WHERE recording_id = $1 AND loss_reason IS NOT NULL) THEN 'capture_loss' ELSE warning END, \
            partial_coverage = partial_coverage OR EXISTS (SELECT 1 FROM mahfel.recording_generations WHERE recording_id = $1 AND loss_reason IS NOT NULL) WHERE id = $1")
            .bind(id).execute(&mut *tx).await?;
        insert_job_in_tx(
            &mut tx,
            &JobEnvelope::recording_finalize(id, generation, OffsetDateTime::now_utc()),
        )
        .await?;
        tx.commit().await?;
        Ok(true)
    }
    pub async fn pending_asset_copies(&self) -> Result<Vec<RecordingAssetCopy>, sqlx::Error> {
        sqlx::query_as("SELECT recording_id, file_id, source_key, archive_key, checksum_sha_256, size_bytes FROM mahfel.recording_assets WHERE copy_status = 'pending' ORDER BY recording_id, file_id LIMIT 100")
            .fetch_all(&self.pool).await
    }
    pub async fn accept_asset_copy(&self, id: Uuid, file: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE mahfel.recording_assets SET copy_status = 'ready' WHERE recording_id = $1 AND file_id = $2 AND copy_status = 'pending'")
            .bind(id).bind(file).execute(&self.pool).await?;
        Ok(())
    }
}

pub struct CaptureObservation {
    pub boot: Uuid,
    pub origin: u64,
    pub capturing: bool,
    pub sealed: bool,
    pub warning: Option<String>,
}
