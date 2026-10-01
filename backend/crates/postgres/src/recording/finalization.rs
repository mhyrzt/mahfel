use crate::{PostgresRecordingStore, stores::sqlx_store_error};
use mahfel_application::{
    jobs::JobEnvelope,
    ports::{BoxFuture, RecordingFinalizationStore, StoreError},
};
use serde_json::Value;
use sqlx::types::Json;
use uuid::Uuid;

impl RecordingFinalizationStore for PostgresRecordingStore {
    fn import_receipt<'a>(
        &'a self,
        receipt: &'a mahfel_application::recording_artifacts::FragmentReceipt,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            self.import_fragment(receipt)
                .await
                .map(|_| ())
                .map_err(sqlx_store_error)
        })
    }
    fn metadata(
        &self,
        recording: Uuid,
        generation: i32,
    ) -> BoxFuture<'_, Result<Option<Value>, StoreError>> {
        Box::pin(async move {
            sqlx::query_scalar::<_, Json<Value>>("SELECT to_jsonb(recording) || jsonb_build_object('session_origin_ns', (extract(epoch FROM session.started_at) * 1000000000)::NUMERIC(20,0)::TEXT) \
                FROM mahfel.recordings recording JOIN mahfel.sessions session ON session.id = recording.session_id \
                WHERE recording.id = $1 AND recording.capture_generation = $2 AND recording.status IN ('finalizing','ready','failed') \
                AND NOT EXISTS (SELECT 1 FROM mahfel.recording_generations WHERE recording_id = $1 AND NOT sealed)")
                .bind(recording).bind(generation).fetch_optional(&self.pool).await.map(|value| value.map(|value| value.0)).map_err(sqlx_store_error)
        })
    }
    fn page(
        &self,
        recording: Uuid,
        section: &'static str,
        offset: i64,
    ) -> BoxFuture<'_, Result<Vec<Value>, StoreError>> {
        Box::pin(async move {
            let query = match section {
                "generations" => {
                    "SELECT to_jsonb(item) || jsonb_build_object('start_ns', (extract(epoch FROM item.started_at - session.started_at) * 1000000000)::NUMERIC(20,0)::TEXT, 'end_ns', (extract(epoch FROM item.ended_at - session.started_at) * 1000000000)::NUMERIC(20,0)::TEXT) FROM mahfel.recording_generations item JOIN mahfel.recordings recording ON recording.id = item.recording_id JOIN mahfel.sessions session ON session.id = recording.session_id WHERE item.recording_id = $1 ORDER BY generation LIMIT 100 OFFSET $2"
                }
                "snapshots" => {
                    "SELECT to_jsonb(item) FROM mahfel.recording_snapshots item WHERE recording_id = $1 ORDER BY generation, snapshot_kind, source_id LIMIT 100 OFFSET $2"
                }
                "events" => {
                    "SELECT to_jsonb(item) || jsonb_build_object('at_ns', (extract(epoch FROM item.occurred_at - session.started_at) * 1000000000)::NUMERIC(20,0)::TEXT) FROM mahfel.recording_events item JOIN mahfel.recordings recording ON recording.id = item.recording_id JOIN mahfel.sessions session ON session.id = recording.session_id WHERE item.recording_id = $1 ORDER BY occurred_at, event_kind, source_id LIMIT 100 OFFSET $2"
                }
                "actors" => {
                    "SELECT jsonb_build_object('actor_id', a.id, 'display_name', COALESCE(u.display_name, p.display_name, 'Participant')) FROM mahfel.actors a LEFT JOIN mahfel.users u ON u.id = a.id LEFT JOIN mahfel.service_principals p ON p.id = a.id WHERE a.id IN (SELECT (payload->>'author_actor_id')::UUID FROM mahfel.recording_events WHERE recording_id = $1 AND event_kind = 'chat' UNION SELECT (receipt->>'actor_id')::UUID FROM mahfel.recording_fragments WHERE recording_id = $1) ORDER BY a.id LIMIT 100 OFFSET $2"
                }
                "assets" => {
                    "SELECT to_jsonb(item) FROM mahfel.recording_assets item WHERE recording_id = $1 ORDER BY file_id LIMIT 100 OFFSET $2"
                }
                "fragments" => {
                    "SELECT receipt FROM mahfel.recording_fragments WHERE recording_id = $1 ORDER BY generation, object_key LIMIT 100 OFFSET $2"
                }
                _ => return Err(StoreError::Unexpected("invalid recording section".into())),
            };
            sqlx::query_scalar::<_, Json<Value>>(query)
                .bind(recording)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map(|values| values.into_iter().map(|value| value.0).collect())
                .map_err(sqlx_store_error)
        })
    }
    fn accept<'a>(
        &'a self,
        job: &'a JobEnvelope,
        token: Uuid,
        manifest: &'a mahfel_application::ports::ManifestAcceptance,
    ) -> BoxFuture<'a, Result<bool, StoreError>> {
        let mahfel_application::ports::ManifestAcceptance {
            recording,
            generation,
            key,
            playback_key,
            partial,
            usable,
        } = manifest;
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let claim: Option<Uuid> = sqlx::query_scalar("SELECT id FROM mahfel.background_jobs WHERE id = $1 AND generation = $2 AND attempt = $3 \
                AND claim_token = $4 AND status = 'running' AND lease_until > clock_timestamp() FOR UPDATE")
                .bind(job.job_id).bind(i32::try_from(job.generation).unwrap_or(0)).bind(i16::try_from(job.attempt).unwrap_or(0)).bind(token).fetch_optional(&mut *tx).await.map_err(sqlx_store_error)?;
            if claim.is_none() {
                return Ok(false);
            }
            let result = sqlx::query("UPDATE mahfel.recordings SET status = CASE WHEN $4 THEN 'ready' ELSE 'failed' END, manifest_key = CASE WHEN $4 THEN COALESCE(manifest_key, $3) ELSE manifest_key END, playback_index_key = CASE WHEN $4 THEN $6 ELSE NULL END, \
                partial_coverage = partial_coverage OR $5, warning = CASE WHEN NOT $4 THEN 'unusable_recording' WHEN $5 THEN 'partial_recording' ELSE NULL END \
                WHERE id = $1 AND capture_generation = $2 AND status = 'finalizing' AND NOT EXISTS (SELECT 1 FROM mahfel.recording_generations WHERE recording_id = $1 AND NOT sealed)")
                .bind(recording).bind(generation).bind(key).bind(usable).bind(partial).bind(playback_key).execute(&mut *tx).await.map_err(sqlx_store_error)?;
            if result.rows_affected() == 1 {
                sqlx::query("UPDATE mahfel.recording_assets SET copy_status = 'ready' WHERE recording_id = $1 AND copy_status = 'pending'")
                    .bind(recording).execute(&mut *tx).await.map_err(sqlx_store_error)?;
            }
            if result.rows_affected() == 1 && *usable {
                super::export::enqueue_export(&mut tx, *recording, key)
                    .await
                    .map_err(sqlx_store_error)?;
            }
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(result.rows_affected() == 1)
        })
    }
}
