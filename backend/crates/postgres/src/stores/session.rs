use crate::DatabaseError;
use crate::stores::{sqlx_store_error, store_error};
use mahfel_application::ports::{BoxFuture, SessionStore, StoreError};
use mahfel_domain::{RoomId, Session, SessionId};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresSessionStore {
    pool: PgPool,
}

impl PostgresSessionStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct SessionRow {
    id: Uuid,
    room_id: Uuid,
    status: String,
    scheduled_start_at: Option<OffsetDateTime>,
    started_at: Option<OffsetDateTime>,
    ended_at: Option<OffsetDateTime>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl SessionStore for PostgresSessionStore {
    fn get(&self, id: SessionId) -> BoxFuture<'_, Result<Option<Session>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, SessionRow>(
                "SELECT id, room_id, status, scheduled_start_at, started_at, ended_at, \
                        created_at, updated_at \
                 FROM mahfel.sessions WHERE id = $1",
            )
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            row.map(|row| Session::try_from(row).map_err(store_error))
                .transpose()
        })
    }

    fn insert<'a>(&'a self, session: &'a Session) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO mahfel.sessions \
                 (id, room_id, status, scheduled_start_at, started_at, ended_at, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
            )
            .bind(session.id.as_uuid())
            .bind(session.room_id.as_uuid())
            .bind(session.status.as_str())
            .bind(session.scheduled_start_at)
            .bind(session.started_at)
            .bind(session.ended_at)
            .bind(session.created_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(())
        })
    }

    fn save<'a>(&'a self, session: &'a Session) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let result = sqlx::query(
                "UPDATE mahfel.sessions \
                 SET status = $2, scheduled_start_at = $3, started_at = $4, ended_at = $5, updated_at = $6 \
                 WHERE id = $1",
            )
            .bind(session.id.as_uuid())
            .bind(session.status.as_str())
            .bind(session.scheduled_start_at)
            .bind(session.started_at)
            .bind(session.ended_at)
            .bind(session.updated_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            if result.rows_affected() == 0 {
                return Err(StoreError::NotFound(session.id.to_string()));
            }
            Ok(())
        })
    }

    fn list_for_room(&self, room_id: RoomId) -> BoxFuture<'_, Result<Vec<Session>, StoreError>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, SessionRow>(
                "SELECT id, room_id, status, scheduled_start_at, started_at, ended_at, \
                        created_at, updated_at \
                 FROM mahfel.sessions WHERE room_id = $1 \
                 ORDER BY created_at DESC, id",
            )
            .bind(room_id.as_uuid())
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            rows.into_iter()
                .map(|row| Session::try_from(row).map_err(store_error))
                .collect()
        })
    }
}

impl TryFrom<SessionRow> for Session {
    type Error = DatabaseError;

    fn try_from(row: SessionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: SessionId::from_uuid(row.id),
            room_id: RoomId::from_uuid(row.room_id),
            status: row.status.as_str().try_into()?,
            scheduled_start_at: row.scheduled_start_at,
            started_at: row.started_at,
            ended_at: row.ended_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
