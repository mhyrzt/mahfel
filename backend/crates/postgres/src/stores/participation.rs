use crate::DatabaseError;
use crate::stores::{sqlx_store_error, store_error};
use mahfel_application::ports::{BoxFuture, ParticipationStore, StoreError};
use mahfel_domain::{
    ActorId, CorrectionReason, IntervalId, ParticipantId, ParticipationInterval, SessionId,
};
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresParticipationStore {
    pool: PgPool,
}

impl PostgresParticipationStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct IntervalRow {
    id: Uuid,
    participant_id: Uuid,
    session_id: Uuid,
    actor_id: Uuid,
    started_at: OffsetDateTime,
    ended_at: Option<OffsetDateTime>,
    superseded_by: Option<Uuid>,
    correction_reason: Option<String>,
    created_at: OffsetDateTime,
}

impl ParticipationStore for PostgresParticipationStore {
    fn open_interval<'a>(
        &'a self,
        interval: &'a ParticipationInterval,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            sqlx::query(
                "INSERT INTO mahfel.participation_intervals \
                 (id, participant_id, session_id, actor_id, started_at, ended_at, \
                  superseded_by, correction_reason, created_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(interval.id.as_uuid())
            .bind(interval.participant_id.as_uuid())
            .bind(interval.session_id.as_uuid())
            .bind(interval.actor_id.as_uuid())
            .bind(interval.started_at)
            .bind(interval.ended_at)
            .bind(interval.superseded_by.map(IntervalId::as_uuid))
            .bind(interval.correction_reason.map(CorrectionReason::as_str))
            .bind(interval.created_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(())
        })
    }

    fn close_open_interval(
        &self,
        participant_id: ParticipantId,
        ended_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<Option<IntervalId>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, (Uuid,)>(
                "UPDATE mahfel.participation_intervals \
                 SET ended_at = $2 \
                 WHERE participant_id = $1 AND ended_at IS NULL \
                 RETURNING id",
            )
            .bind(participant_id.as_uuid())
            .bind(ended_at)
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(row.map(|(id,)| IntervalId::from_uuid(id)))
        })
    }

    fn close_open_intervals_for_session(
        &self,
        session_id: SessionId,
        ended_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<u64, StoreError>> {
        Box::pin(async move {
            let result = sqlx::query(
                "UPDATE mahfel.participation_intervals \
                 SET ended_at = $2 \
                 WHERE session_id = $1 AND ended_at IS NULL",
            )
            .bind(session_id.as_uuid())
            .bind(ended_at)
            .execute(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            Ok(result.rows_affected())
        })
    }

    fn get(
        &self,
        id: IntervalId,
    ) -> BoxFuture<'_, Result<Option<ParticipationInterval>, StoreError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, IntervalRow>(
                "SELECT id, participant_id, session_id, actor_id, started_at, ended_at, \
                        superseded_by, correction_reason, created_at \
                 FROM mahfel.participation_intervals WHERE id = $1",
            )
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            row.map(|row| ParticipationInterval::try_from(row).map_err(store_error))
                .transpose()
        })
    }

    fn list_for_participant(
        &self,
        participant_id: ParticipantId,
    ) -> BoxFuture<'_, Result<Vec<ParticipationInterval>, StoreError>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, IntervalRow>(
                "SELECT id, participant_id, session_id, actor_id, started_at, ended_at, \
                        superseded_by, correction_reason, created_at \
                 FROM mahfel.participation_intervals \
                 WHERE participant_id = $1 ORDER BY started_at, id",
            )
            .bind(participant_id.as_uuid())
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            rows.into_iter()
                .map(|row| ParticipationInterval::try_from(row).map_err(store_error))
                .collect()
        })
    }

    fn list_for_session(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Vec<ParticipationInterval>, StoreError>> {
        Box::pin(async move {
            let rows = sqlx::query_as::<_, IntervalRow>(
                "SELECT id, participant_id, session_id, actor_id, started_at, ended_at, \
                        superseded_by, correction_reason, created_at \
                 FROM mahfel.participation_intervals \
                 WHERE session_id = $1 ORDER BY started_at, id",
            )
            .bind(session_id.as_uuid())
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_store_error)?;
            rows.into_iter()
                .map(|row| ParticipationInterval::try_from(row).map_err(store_error))
                .collect()
        })
    }

    fn supersede_interval<'a>(
        &'a self,
        original: &'a ParticipationInterval,
        replacement: &'a ParticipationInterval,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(sqlx_store_error)?;

            // Insert the replacement first: the original's `superseded_by`
            // references it, so the row must exist before the mark lands.
            sqlx::query(
                "INSERT INTO mahfel.participation_intervals \
                 (id, participant_id, session_id, actor_id, started_at, ended_at, \
                  superseded_by, correction_reason, created_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, NULL, NULL, $7)",
            )
            .bind(replacement.id.as_uuid())
            .bind(replacement.participant_id.as_uuid())
            .bind(replacement.session_id.as_uuid())
            .bind(replacement.actor_id.as_uuid())
            .bind(replacement.started_at)
            .bind(replacement.ended_at)
            .bind(replacement.created_at)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;

            let marked = sqlx::query(
                "UPDATE mahfel.participation_intervals \
                 SET superseded_by = $2, correction_reason = $3, ended_at = $4 \
                 WHERE id = $1 AND superseded_by IS NULL",
            )
            .bind(original.id.as_uuid())
            .bind(replacement.id.as_uuid())
            .bind(original.correction_reason.map(CorrectionReason::as_str))
            .bind(original.ended_at)
            .execute(&mut *tx)
            .await
            .map_err(sqlx_store_error)?;

            // Someone already corrected this interval; refuse rather than
            // recording a second competing replacement.
            if marked.rows_affected() == 0 {
                return Err(StoreError::Conflict(
                    "interval already superseded".to_owned(),
                ));
            }

            tx.commit().await.map_err(sqlx_store_error)?;
            Ok(())
        })
    }
}

impl TryFrom<IntervalRow> for ParticipationInterval {
    type Error = DatabaseError;

    fn try_from(row: IntervalRow) -> Result<Self, Self::Error> {
        let correction_reason = row
            .correction_reason
            .as_deref()
            .map(CorrectionReason::try_from)
            .transpose()?;
        Ok(Self {
            id: IntervalId::from_uuid(row.id),
            participant_id: ParticipantId::from_uuid(row.participant_id),
            session_id: SessionId::from_uuid(row.session_id),
            actor_id: ActorId::from_uuid(row.actor_id),
            started_at: row.started_at,
            ended_at: row.ended_at,
            superseded_by: row.superseded_by.map(IntervalId::from_uuid),
            correction_reason,
            created_at: row.created_at,
        })
    }
}
