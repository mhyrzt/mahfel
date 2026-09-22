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
            let session_id = match target {
                AdmissionTarget::Session(session_id) => Some(session_id.as_uuid()),
                AdmissionTarget::Room(_) => None,
            };
            let query = format!(
                "SELECT {SELECT_COLUMNS} FROM mahfel.admission_requests \
                 WHERE session_id IS NOT DISTINCT FROM $1 \
                   AND actor_id = $2 AND status = 'pending'"
            );
            let row = sqlx::query_as::<_, RequestRow>(sqlx::AssertSqlSafe(query))
                .bind(session_id)
                .bind(actor_id.as_uuid())
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
            let session_id = match target {
                AdmissionTarget::Session(session_id) => Some(session_id.as_uuid()),
                AdmissionTarget::Room(_) => None,
            };
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM mahfel.admission_requests \
                 WHERE session_id IS NOT DISTINCT FROM $1 AND status = 'pending'",
            )
            .bind(session_id)
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
