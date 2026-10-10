use crate::stores::sqlx_store_error;
use mahfel_application::ports::{AdmissionStore, BoxFuture, StoreError};
use mahfel_domain::{
    ActorId, AdmissionOrigin, AdmissionRequest, AdmissionRequestId, AdmissionRequestStatus,
    AdmissionTarget, RoomId, SessionId,
};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresAdmissionStore {
    pool: PgPool,
}

impl PostgresAdmissionStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct RequestRow {
    id: Uuid,
    session_id: Option<Uuid>,
    room_id: Uuid,
    actor_id: Uuid,
    status: String,
    origin: String,
    requested_at: OffsetDateTime,
    decided_at: Option<OffsetDateTime>,
    decided_by: Option<Uuid>,
}

impl RequestRow {
    fn into_request(self) -> Result<AdmissionRequest, StoreError> {
        let status: AdmissionRequestStatus =
            self.status
                .as_str()
                .try_into()
                .map_err(|error: mahfel_domain::InvalidStatus| {
                    StoreError::Unexpected(error.to_string())
                })?;
        let origin: AdmissionOrigin =
            self.origin
                .as_str()
                .try_into()
                .map_err(|error: mahfel_domain::InvalidStatus| {
                    StoreError::Unexpected(error.to_string())
                })?;
        // `room_id` is always present; a session id means the request is scoped
        // to that one session rather than to the room.
        let target = match self.session_id {
            Some(session_id) => AdmissionTarget::Session(SessionId::from_uuid(session_id)),
            None => AdmissionTarget::Room(RoomId::from_uuid(self.room_id)),
        };
        Ok(AdmissionRequest {
            id: AdmissionRequestId::from_uuid(self.id),
            target,
            actor_id: ActorId::from_uuid(self.actor_id),
            status,
            origin,
            requested_at: self.requested_at,
            decided_at: self.decided_at,
            decided_by: self.decided_by.map(ActorId::from_uuid),
        })
    }
}

const SELECT_COLUMNS: &str = "id, session_id, room_id, actor_id, status, origin, requested_at, \
                              decided_at, decided_by";

impl AdmissionStore for PostgresAdmissionStore {
    fn insert<'a>(
        &'a self,
        request: &'a AdmissionRequest,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            // A session-scoped row still records its room, because room_id is
            // NOT NULL and authorization resolves against it either way.
            let (session_id, room_id) = match request.target {
                AdmissionTarget::Session(session_id) => {
                    let room_id = sqlx::query_scalar::<_, Uuid>(
                        "SELECT room_id FROM mahfel.sessions WHERE id = $1",
                    )
                    .bind(session_id.as_uuid())
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(sqlx_store_error)?
                    .ok_or_else(|| {
                        StoreError::MissingReference("session has no room".to_owned())
                    })?;
                    (Some(session_id.as_uuid()), room_id)
                }
                AdmissionTarget::Room(room_id) => (None, room_id.as_uuid()),
            };
            sqlx::query(
                "INSERT INTO mahfel.admission_requests \
                 (id, session_id, room_id, actor_id, status, origin, requested_at, decided_at, \
                  decided_by) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(request.id.as_uuid())
            .bind(session_id)
            .bind(room_id)
            .bind(request.actor_id.as_uuid())
            .bind(request.status.as_str())
            .bind(request.origin.as_str())
            .bind(request.requested_at)
            .bind(request.decided_at)
            .bind(request.decided_by.map(ActorId::as_uuid))
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(())
        })
    }

    fn get_pending(
        &self,
        target: AdmissionTarget,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<AdmissionRequest>, StoreError>> {
        Box::pin(async move {
            let (session_id, room_id) = match target {
                AdmissionTarget::Session(session_id) => (Some(session_id.as_uuid()), None),
                AdmissionTarget::Room(room_id) => (None, Some(room_id.as_uuid())),
            };
            let query = format!(
                "SELECT {SELECT_COLUMNS} FROM mahfel.admission_requests \
                 WHERE session_id IS NOT DISTINCT FROM $1 \
                   AND actor_id = $2 AND status = 'pending' \
                   AND ($3::uuid IS NULL OR room_id = $3)"
            );
            let row = sqlx::query_as::<_, RequestRow>(sqlx::AssertSqlSafe(query))
                .bind(session_id)
                .bind(actor_id.as_uuid())
                .bind(room_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            row.map(RequestRow::into_request).transpose()
        })
    }

    fn get(
        &self,
        id: AdmissionRequestId,
    ) -> BoxFuture<'_, Result<Option<AdmissionRequest>, StoreError>> {
        Box::pin(async move {
            let query =
                format!("SELECT {SELECT_COLUMNS} FROM mahfel.admission_requests WHERE id = $1");
            let row = sqlx::query_as::<_, RequestRow>(sqlx::AssertSqlSafe(query))
                .bind(id.as_uuid())
                .fetch_optional(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            row.map(RequestRow::into_request).transpose()
        })
    }

    fn list_pending(
        &self,
        session_id: SessionId,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<AdmissionRequest>, StoreError>> {
        Box::pin(async move {
            let query = format!(
                "SELECT {SELECT_COLUMNS} FROM mahfel.admission_requests \
                 WHERE session_id = $1 AND status = 'pending' \
                 ORDER BY requested_at, id LIMIT $2"
            );
            let rows = sqlx::query_as::<_, RequestRow>(sqlx::AssertSqlSafe(query))
                .bind(session_id.as_uuid())
                .bind(i64::from(limit))
                .fetch_all(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            rows.into_iter().map(RequestRow::into_request).collect()
        })
    }

    fn list_pending_for_room(
        &self,
        room_id: RoomId,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<AdmissionRequest>, StoreError>> {
        Box::pin(async move {
            let query = format!(
                "SELECT {SELECT_COLUMNS} FROM mahfel.admission_requests \
                 WHERE room_id = $1 AND session_id IS NULL AND status = 'pending' \
                 ORDER BY requested_at, id LIMIT $2"
            );
            let rows = sqlx::query_as::<_, RequestRow>(sqlx::AssertSqlSafe(query))
                .bind(room_id.as_uuid())
                .bind(i64::from(limit))
                .fetch_all(&self.pool)
                .await
                .map_err(sqlx_store_error)?;
            rows.into_iter().map(RequestRow::into_request).collect()
        })
    }

    fn count_pending(&self, target: AdmissionTarget) -> BoxFuture<'_, Result<u64, StoreError>> {
        Box::pin(async move {
            let (session_id, room_id) = match target {
                AdmissionTarget::Session(session_id) => (Some(session_id.as_uuid()), None),
                AdmissionTarget::Room(room_id) => (None, Some(room_id.as_uuid())),
            };
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM mahfel.admission_requests \
                 WHERE session_id IS NOT DISTINCT FROM $1 AND status = 'pending' \
                   AND ($2::uuid IS NULL OR room_id = $2)",
            )
            .bind(session_id)
            .bind(room_id)
            .fetch_one(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(u64::try_from(count).unwrap_or(0))
        })
    }

    fn decide_if_pending(
        &self,
        id: AdmissionRequestId,
        status: AdmissionRequestStatus,
        decided_by: Option<ActorId>,
        decided_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            // Conditional so a duplicated decision cannot both report success,
            // and the first decision is never overwritten.
            let result = sqlx::query(
                "UPDATE mahfel.admission_requests \
                 SET status = $2, decided_at = $3, decided_by = $4 \
                 WHERE id = $1 AND status = 'pending'",
            )
            .bind(id.as_uuid())
            .bind(status.as_str())
            .bind(decided_at)
            .bind(decided_by.map(ActorId::as_uuid))
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(result.rows_affected() == 1)
        })
    }

    fn approve_pending(
        &self,
        id: AdmissionRequestId,
        actor: ActorId,
        now: OffsetDateTime,
    ) -> BoxFuture<
        '_,
        Result<
            Option<mahfel_application::admission::AdmissionGrant>,
            mahfel_application::admission::AdmissionOperationError,
        >,
    > {
        Box::pin(async move {
            use mahfel_application::admission::{AdmissionGrant, AdmissionOperationError};
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;
            let request = sqlx::query_as::<_, RequestRow>(
                "SELECT id, session_id, room_id, actor_id, status, origin, requested_at, decided_at, decided_by \
                 FROM mahfel.admission_requests WHERE id = $1",
            ).bind(id.as_uuid()).fetch_optional(&mut *tx).await.map_err(sqlx_store_error)?;
            let Some(request) = request else {
                return Ok(None);
            };
            let room_status = sqlx::query_scalar::<_, String>(
                "SELECT status FROM mahfel.rooms WHERE id = $1 FOR UPDATE",
            )
            .bind(request.room_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            if room_status.as_deref() != Some("active") {
                return Err(AdmissionOperationError::NotFound);
            }
            let manager = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (SELECT 1 FROM mahfel.room_memberships \
                 WHERE room_id = $1 AND actor_id = $2 AND member_role = 'manager')",
            )
            .bind(request.room_id)
            .bind(actor.as_uuid())
            .fetch_one(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;
            if !manager {
                return Err(AdmissionOperationError::Forbidden);
            }
            if let Some(session_id) = request.session_id {
                let status = sqlx::query_scalar::<_, String>(
                    "SELECT status FROM mahfel.sessions WHERE id = $1 FOR UPDATE",
                )
                .bind(session_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(sqlx_store_error)?;
                if status.as_deref() != Some("live") {
                    return Err(AdmissionOperationError::NotLive);
                }
            }
            let decision = sqlx::query(
                "UPDATE mahfel.admission_requests SET status = 'admitted', decided_at = $2, decided_by = $3 \
                 WHERE id = $1 AND status = 'pending'",
            ).bind(id.as_uuid()).bind(now).bind(actor.as_uuid()).execute(&mut *tx).await.map_err(sqlx_store_error)?;
            if decision.rows_affected() != 1 {
                return Ok(None);
            }
            let grant = if let Some(session_id) = request.session_id {
                let row = sqlx::query_as::<_, super::participant::ParticipantRow>(
                    "INSERT INTO mahfel.session_participants \
                     (id, session_id, actor_id, status, joined_at, created_at, updated_at) \
                     VALUES ($1, $2, $3, 'active', $4, $4, $4) \
                     ON CONFLICT (session_id, actor_id) WHERE status = 'active' \
                     DO UPDATE SET updated_at = mahfel.session_participants.updated_at \
                     RETURNING id, session_id, actor_id, status, joined_at, left_at, \
                     muted_at, mute_kind, muted_by, created_at, updated_at",
                )
                .bind(Uuid::new_v4())
                .bind(session_id)
                .bind(request.actor_id)
                .bind(now)
                .fetch_one(&mut *tx)
                .await
                .map_err(sqlx_store_error)?;
                let participant =
                    mahfel_domain::Participant::try_from(row).map_err(super::store_error)?;
                sqlx::query(
                    "INSERT INTO mahfel.participation_intervals \
                     (id, participant_id, session_id, actor_id, started_at, created_at) \
                     VALUES ($1, $2, $3, $4, $5, $5) \
                     ON CONFLICT (participant_id) WHERE ended_at IS NULL DO NOTHING",
                )
                .bind(Uuid::new_v4())
                .bind(participant.id.as_uuid())
                .bind(session_id)
                .bind(request.actor_id)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(sqlx_store_error)?;
                AdmissionGrant::Participant(participant)
            } else {
                sqlx::query(
                    "INSERT INTO mahfel.room_memberships \
                     (room_id, actor_id, member_role, created_at, updated_at) \
                     VALUES ($1, $2, 'member', $3, $3) ON CONFLICT (room_id, actor_id) DO NOTHING",
                )
                .bind(request.room_id)
                .bind(request.actor_id)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(sqlx_store_error)?;
                AdmissionGrant::Membership {
                    room_id: RoomId::from_uuid(request.room_id),
                }
            };
            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(Some(grant))
        })
    }

    fn expire_pending_for_session(
        &self,
        session_id: SessionId,
        expired_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<u64, StoreError>> {
        Box::pin(async move {
            // Room-scoped requests are deliberately untouched: they ask to join
            // the room, which outlives any one session.
            let result = sqlx::query(
                "UPDATE mahfel.admission_requests \
                 SET status = 'expired', decided_at = $2, decided_by = NULL \
                 WHERE session_id = $1 AND status = 'pending'",
            )
            .bind(session_id.as_uuid())
            .bind(expired_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(result.rows_affected())
        })
    }
}
