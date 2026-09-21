use crate::DatabaseError;
use crate::stores::{sqlx_store_error, store_error};
use mahfel_application::ports::{BoxFuture, ParticipantStore, StoreError};
use mahfel_domain::{ActorId, Participant, ParticipantId, ParticipantStatus, SessionId};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresParticipantStore {
    pool: PgPool,
}

impl PostgresParticipantStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct ParticipantRow {
    id: Uuid,
    session_id: Uuid,
    actor_id: Uuid,
    status: String,
    joined_at: OffsetDateTime,
    left_at: Option<OffsetDateTime>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl ParticipantStore for PostgresParticipantStore {
    fn get(&self, id: ParticipantId) -> BoxFuture<'_, Result<Option<Participant>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, ParticipantRow>(
                "SELECT id, session_id, actor_id, status, joined_at, left_at, \
                        created_at, updated_at \
                 FROM mahfel.session_participants WHERE id = $1",
            )
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            row.map(|row| Participant::try_from(row).map_err(store_error))
                .transpose()
        })
    }

    fn get_active(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<Participant>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, ParticipantRow>(
                "SELECT id, session_id, actor_id, status, joined_at, left_at, \
                        created_at, updated_at \
                 FROM mahfel.session_participants \
                 WHERE session_id = $1 AND actor_id = $2 AND status = 'active'",
            )
            .bind(session_id.as_uuid())
            .bind(actor_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            row.map(|row| Participant::try_from(row).map_err(store_error))
                .transpose()
        })
    }

    fn insert<'a>(&'a self, participant: &'a Participant) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO mahfel.session_participants \
                 (id, session_id, actor_id, status, joined_at, left_at, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
            )
            .bind(participant.id.as_uuid())
            .bind(participant.session_id.as_uuid())
            .bind(participant.actor_id.as_uuid())
            .bind(participant.status.as_str())
            .bind(participant.joined_at)
            .bind(participant.left_at)
            .bind(participant.created_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(())
        })
    }

    fn save<'a>(&'a self, participant: &'a Participant) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let result = sqlx::query(
                "UPDATE mahfel.session_participants \
                 SET status = $2, left_at = $3, updated_at = $4 \
                 WHERE id = $1",
            )
            .bind(participant.id.as_uuid())
            .bind(participant.status.as_str())
            .bind(participant.left_at)
            .bind(participant.updated_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            if result.rows_affected() == 0 {
                return Err(StoreError::NotFound(participant.id.to_string()));
            }
            Ok(())
        })
    }

    fn list_for_session(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Vec<Participant>, StoreError>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, ParticipantRow>(
                "SELECT id, session_id, actor_id, status, joined_at, left_at, \
                        created_at, updated_at \
                 FROM mahfel.session_participants \
                 WHERE session_id = $1 ORDER BY joined_at, id",
            )
            .bind(session_id.as_uuid())
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_store_error)?;

            rows.into_iter()
                .map(|row| Participant::try_from(row).map_err(store_error))
                .collect()
        })
    }
}

impl TryFrom<ParticipantRow> for Participant {
    type Error = DatabaseError;

    fn try_from(row: ParticipantRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ParticipantId::from_uuid(row.id),
            session_id: SessionId::from_uuid(row.session_id),
            actor_id: ActorId::from_uuid(row.actor_id),
            status: ParticipantStatus::try_from(row.status.as_str())?,
            joined_at: row.joined_at,
            left_at: row.left_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
