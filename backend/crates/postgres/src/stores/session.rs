use crate::DatabaseError;
use crate::stores::{sqlx_store_error, store_error};
use mahfel_application::ports::{BoxFuture, SessionDetailsFields, SessionStore, StoreError};
use mahfel_domain::{
    AdmissionPolicy, RoomId, Session, SessionDurationMinutes, SessionId, SessionTitle,
    SessionVisibility,
};
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

    /// Ensure a scheduled development session exists without resetting its
    /// lifecycle state on subsequent seed runs.
    pub async fn seed_development_session(&self, session: &Session) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO mahfel.sessions \
             (id, room_id, status, title, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $5) ON CONFLICT (id) DO NOTHING",
        )
        .bind(session.id.as_uuid())
        .bind(session.room_id.as_uuid())
        .bind(session.status.as_str())
        .bind(session.title.as_ref().map(SessionTitle::as_str))
        .bind(session.created_at)
        .execute(&self.pool)
        .await
        .map_err(sqlx_store_error)?;
        let room_id: Uuid = sqlx::query_scalar("SELECT room_id FROM mahfel.sessions WHERE id = $1")
            .bind(session.id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
        if room_id != session.room_id.as_uuid() {
            return Err(StoreError::Conflict(format!(
                "seed session {} belongs to another room",
                session.id
            )));
        }
        Ok(())
    }
}

#[derive(FromRow)]
struct SessionRow {
    id: Uuid,
    room_id: Uuid,
    status: String,
    auto_record: bool,
    visibility: Option<String>,
    admission: Option<String>,
    title: Option<String>,
    scheduled_start_at: Option<OffsetDateTime>,
    scheduled_duration_minutes: Option<i32>,
    started_at: Option<OffsetDateTime>,
    ended_at: Option<OffsetDateTime>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl SessionStore for PostgresSessionStore {
    fn get(&self, id: SessionId) -> BoxFuture<'_, Result<Option<Session>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, SessionRow>(
                "SELECT id, room_id, status, auto_record, visibility, admission, title, scheduled_start_at, \
                        scheduled_duration_minutes, \
                        started_at, ended_at, created_at, updated_at \
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
                 (id, room_id, status, visibility, admission, title, scheduled_start_at, \
                  scheduled_duration_minutes, started_at, ended_at, created_at, updated_at, auto_record) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11, $12)",
            )
            .bind(session.id.as_uuid())
            .bind(session.room_id.as_uuid())
            .bind(session.status.as_str())
            .bind(session.visibility.map(SessionVisibility::as_str))
            .bind(session.admission.map(AdmissionPolicy::as_str))
            .bind(session.title.as_ref().map(SessionTitle::as_str))
            .bind(session.scheduled_start_at)
            .bind(
                session
                    .scheduled_duration_minutes
                    .map(|value| value.get() as i32),
            )
            .bind(session.started_at)
            .bind(session.ended_at)
            .bind(session.created_at)
            .bind(session.auto_record)
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
                 SET status = $2, visibility = $3, admission = $4, title = $5, \
                     scheduled_start_at = $6, scheduled_duration_minutes = $7, \
                     started_at = $8, ended_at = $9, updated_at = $10 \
                 WHERE id = $1",
            )
            .bind(session.id.as_uuid())
            .bind(session.status.as_str())
            .bind(session.visibility.map(SessionVisibility::as_str))
            .bind(session.admission.map(AdmissionPolicy::as_str))
            .bind(session.title.as_ref().map(SessionTitle::as_str))
            .bind(session.scheduled_start_at)
            .bind(
                session
                    .scheduled_duration_minutes
                    .map(|value| value.get() as i32),
            )
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

    fn save_details<'a>(
        &'a self,
        session: &'a Session,
        fields: SessionDetailsFields,
    ) -> BoxFuture<'a, Result<Session, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, SessionRow>(
                "UPDATE mahfel.sessions \
                 SET title = CASE WHEN $2 THEN $3 ELSE title END, \
                     scheduled_start_at = CASE WHEN $4 THEN $5 ELSE scheduled_start_at END, \
                     scheduled_duration_minutes = \
                         CASE WHEN $6 THEN $7 ELSE scheduled_duration_minutes END, \
                     auto_record = CASE WHEN $9 THEN $10 ELSE auto_record END, \
                     updated_at = GREATEST(updated_at, $8) \
                 WHERE id = $1 AND (NOT $9 OR status = 'scheduled') \
                 RETURNING id, room_id, status, auto_record, visibility, admission, title, \
                           scheduled_start_at, scheduled_duration_minutes, started_at, \
                           ended_at, created_at, updated_at",
            )
            .bind(session.id.as_uuid())
            .bind(fields.title)
            .bind(session.title.as_ref().map(SessionTitle::as_str))
            .bind(fields.scheduled_start_at)
            .bind(session.scheduled_start_at)
            .bind(fields.scheduled_duration_minutes)
            .bind(
                session
                    .scheduled_duration_minutes
                    .map(|value| value.get() as i32),
            )
            .bind(session.updated_at)
            .bind(fields.auto_record)
            .bind(session.auto_record)
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?
            .ok_or_else(|| {
                if fields.auto_record {
                    StoreError::Conflict("recording preference locked".into())
                } else {
                    StoreError::NotFound(session.id.to_string())
                }
            })?;
            Session::try_from(row).map_err(store_error)
        })
    }

    fn list_for_room(&self, room_id: RoomId) -> BoxFuture<'_, Result<Vec<Session>, StoreError>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, SessionRow>(
                "SELECT id, room_id, status, auto_record, visibility, admission, title, scheduled_start_at, \
                        scheduled_duration_minutes, \
                        started_at, ended_at, created_at, updated_at \
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
            auto_record: row.auto_record,
            visibility: row
                .visibility
                .as_deref()
                .map(SessionVisibility::try_from)
                .transpose()?,
            admission: row
                .admission
                .as_deref()
                .map(AdmissionPolicy::try_from)
                .transpose()?,
            title: row
                .title
                .map(SessionTitle::new)
                .transpose()
                .map_err(|_| DatabaseError::InvalidStoredValue("session title"))?,
            scheduled_start_at: row.scheduled_start_at,
            scheduled_duration_minutes: row
                .scheduled_duration_minutes
                .map(|value| {
                    u32::try_from(value)
                        .ok()
                        .and_then(|value| SessionDurationMinutes::new(value).ok())
                        .ok_or(DatabaseError::InvalidStoredValue("session duration"))
                })
                .transpose()?,
            started_at: row.started_at,
            ended_at: row.ended_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
