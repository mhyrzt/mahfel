use crate::stores::{sqlx_store_error, store_error};
use mahfel_application::ports::{BoxFuture, RecordingStore, StoreError};
use mahfel_domain::{ActorId, Recording, RecordingAction, RecordingId, SessionId};
use sqlx::{FromRow, PgPool, types::Json};
use time::OffsetDateTime;
use uuid::Uuid;

const COLUMNS: &str = "id, session_id, status, revision, capture_generation, initiated_by, started_at, stopped_at, warning, partial_coverage AS partial, manifest_key";

#[derive(Clone)]
pub struct PostgresRecordingStore {
    pub(crate) pool: PgPool,
}

impl PostgresRecordingStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct RecordingRow {
    id: Uuid,
    session_id: Uuid,
    status: String,
    revision: i64,
    capture_generation: i32,
    initiated_by: Option<Uuid>,
    started_at: OffsetDateTime,
    stopped_at: Option<OffsetDateTime>,
    warning: Option<String>,
    partial: bool,
    manifest_key: Option<String>,
}

impl RecordingRow {
    fn into_recording(self) -> Result<Recording, StoreError> {
        Ok(Recording {
            id: RecordingId::from_uuid(self.id),
            session_id: SessionId::from_uuid(self.session_id),
            status: self.status.as_str().try_into().map_err(store_error)?,
            revision: self.revision,
            capture_generation: self.capture_generation,
            initiated_by: self.initiated_by.map(ActorId::from_uuid),
            started_at: self.started_at,
            stopped_at: self.stopped_at,
            warning: self.warning,
            partial: self.partial,
            manifest_key: self.manifest_key,
        })
    }
}

impl RecordingStore for PostgresRecordingStore {
    fn expire_capture(&self, session: SessionId) -> BoxFuture<'_, Result<(), StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            sqlx::query("SELECT id FROM mahfel.sessions WHERE id = $1 FOR UPDATE")
                .bind(session.as_uuid())
                .execute(&mut *tx)
                .await
                .map_err(sqlx_store_error)?;
            let records: Vec<(Uuid,i32)> = sqlx::query_as("SELECT id, capture_generation FROM mahfel.recordings WHERE session_id = $1 AND status = 'stopping' AND stopped_at < clock_timestamp() - INTERVAL '29 seconds' FOR UPDATE")
                .bind(session.as_uuid()).fetch_all(&mut *tx).await.map_err(sqlx_store_error)?;
            for (id, generation) in records {
                sqlx::query("UPDATE mahfel.recording_generations SET sealed = TRUE, ended_at = COALESCE(ended_at, clock_timestamp()), loss_reason = COALESCE(loss_reason, 'seal_timeout') WHERE recording_id = $1 AND NOT sealed")
                    .bind(id).execute(&mut *tx).await.map_err(sqlx_store_error)?;
                sqlx::query("UPDATE mahfel.recordings SET status = 'finalizing', revision = revision + 1, partial_coverage = TRUE, warning = 'seal_timeout' WHERE id = $1")
                    .bind(id).execute(&mut *tx).await.map_err(sqlx_store_error)?;
                crate::insert_job_in_tx(
                    &mut tx,
                    &mahfel_application::jobs::JobEnvelope::recording_finalize(
                        id,
                        generation,
                        OffsetDateTime::now_utc(),
                    ),
                )
                .await
                .map_err(sqlx_store_error)?;
            }
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(())
        })
    }
    fn capture_pending(&self, session: SessionId) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM mahfel.recordings WHERE session_id = $1 AND status = 'stopping')")
                .bind(session.as_uuid()).fetch_one(&self.pool).await.map_err(sqlx_store_error)
        })
    }
    fn can_access(
        &self,
        session: SessionId,
        actor: ActorId,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM mahfel.sessions session JOIN mahfel.rooms room ON room.id = session.room_id \
                 WHERE session.id = $1 AND room.status = 'active' AND ( \
                   EXISTS (SELECT 1 FROM mahfel.room_memberships member WHERE member.room_id = room.id AND member.actor_id = $2) \
                   OR EXISTS (SELECT 1 FROM mahfel.session_participants participant WHERE participant.session_id = session.id AND participant.actor_id = $2 AND participant.status = 'active') \
                   OR (room.visibility = 'public' AND COALESCE(session.visibility, room.visibility) = 'public')) \
                 AND NOT EXISTS (SELECT 1 FROM mahfel.blocks block WHERE block.actor_id = $2 \
                   AND (block.room_id = room.id OR block.session_id = session.id) AND block.lifted_at IS NULL \
                   AND (block.expires_at IS NULL OR block.expires_at > clock_timestamp())))"
            ).bind(session.as_uuid()).bind(actor.as_uuid()).fetch_one(&self.pool).await.map_err(sqlx_store_error)
        })
    }

    fn list(&self, session: SessionId) -> BoxFuture<'_, Result<Vec<Recording>, StoreError>> {
        Box::pin(async move {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM mahfel.sessions WHERE id = $1)")
                    .bind(session.as_uuid())
                    .fetch_one(&self.pool)
                    .await
                    .map_err(sqlx_store_error)?;
            if !exists {
                return Err(StoreError::NotFound("session".into()));
            }
            let sql = format!(
                "SELECT {COLUMNS} FROM mahfel.recordings WHERE session_id = $1 AND deleted_at IS NULL ORDER BY started_at DESC, id LIMIT 100"
            );
            sqlx::query_as::<_, RecordingRow>(sqlx::AssertSqlSafe(sql))
                .bind(session.as_uuid())
                .fetch_all(&self.pool)
                .await
                .map_err(sqlx_store_error)?
                .into_iter()
                .map(RecordingRow::into_recording)
                .collect()
        })
    }

    fn command(
        &self,
        session: SessionId,
        actor: ActorId,
        command_id: Uuid,
        action: RecordingAction,
    ) -> BoxFuture<'_, Result<Recording, StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let status: String =
                sqlx::query_scalar("SELECT status FROM mahfel.sessions WHERE id = $1 FOR UPDATE")
                    .bind(session.as_uuid())
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(sqlx_store_error)?
                    .ok_or_else(|| StoreError::NotFound("session".into()))?;
            let replay: Option<(Uuid, String, Json<Recording>)> = sqlx::query_as(
                "SELECT actor_id, action_kind, response FROM mahfel.recording_commands WHERE session_id = $1 AND command_id = $2"
            ).bind(session.as_uuid()).bind(command_id).fetch_optional(&mut *tx).await.map_err(sqlx_store_error)?;
            if let Some((original_actor, original_action, response)) = replay {
                if original_actor != actor.as_uuid() || original_action != action.as_str() {
                    return Err(StoreError::Conflict("command id reused".into()));
                }
                return Ok(response.0);
            }
            if status != "live" {
                return Err(StoreError::Conflict("session is not live".into()));
            }
            let sql = format!(
                "SELECT {COLUMNS} FROM mahfel.recordings WHERE session_id = $1 AND status IN ('starting','recording','paused','stopping') FOR UPDATE"
            );
            let current = sqlx::query_as::<_, RecordingRow>(sqlx::AssertSqlSafe(sql))
                .bind(session.as_uuid())
                .fetch_optional(&mut *tx)
                .await
                .map_err(sqlx_store_error)?;
            let recording_id;
            if action == RecordingAction::Start {
                if current.is_some() {
                    return Err(StoreError::Conflict("capture already exists".into()));
                }
                recording_id = Uuid::new_v4();
                sqlx::query("INSERT INTO mahfel.recordings (id, session_id, status, initiated_by) VALUES ($1,$2,'starting',$3)")
                    .bind(recording_id).bind(session.as_uuid()).bind(actor.as_uuid()).execute(&mut *tx).await.map_err(sqlx_store_error)?;
            } else {
                let recording = current
                    .ok_or_else(|| StoreError::Conflict("no unfinished recording".into()))?
                    .into_recording()?;
                let next = recording
                    .status
                    .transition(action)
                    .map_err(|_| StoreError::Conflict("invalid transition".into()))?;
                recording_id = recording.id.as_uuid();
                sqlx::query("UPDATE mahfel.recordings SET status = $2, revision = revision + 1, \
                             capture_generation = capture_generation + CASE WHEN $3 THEN 1 ELSE 0 END, \
                             stopped_at = CASE WHEN $4 THEN COALESCE(stopped_at, clock_timestamp()) ELSE stopped_at END \
                             WHERE id = $1")
                    .bind(recording_id).bind(next.as_str()).bind(action == RecordingAction::Resume)
                    .bind(action == RecordingAction::Stop).execute(&mut *tx).await.map_err(sqlx_store_error)?;
            }
            let sql = format!("SELECT {COLUMNS} FROM mahfel.recordings WHERE id = $1");
            let recording = sqlx::query_as::<_, RecordingRow>(sqlx::AssertSqlSafe(sql))
                .bind(recording_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(sqlx_store_error)?
                .into_recording()?;
            sqlx::query("INSERT INTO mahfel.recording_audit (recording_id, actor_id, action_kind, reason, revision) VALUES ($1,$2,$3,'manual',$4)")
                .bind(recording_id).bind(actor.as_uuid()).bind(action.as_str()).bind(recording.revision)
                .execute(&mut *tx).await.map_err(sqlx_store_error)?;
            sqlx::query("INSERT INTO mahfel.recording_commands (session_id, command_id, actor_id, action_kind, response) VALUES ($1,$2,$3,$4,$5)")
                .bind(session.as_uuid()).bind(command_id).bind(actor.as_uuid()).bind(action.as_str())
                .bind(Json(&recording)).execute(&mut *tx).await.map_err(sqlx_store_error)?;
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(recording)
        })
    }
}
