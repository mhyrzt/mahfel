use super::{JobAttempt, JobRecord, JobReplay, JobStore, model::StoredJobRow};
use mahfel_application::jobs::JobEnvelope;
use time::OffsetDateTime;
use uuid::Uuid;

impl JobStore {
    pub async fn inspect(&self, id: Uuid) -> Result<Option<JobRecord>, sqlx::Error> {
        let row: Option<StoredJobRow> = sqlx::query_as(
            "SELECT id, job_type, schema_version, payload, idempotency_key, correlation_id, \
             traceparent, tracestate, created_at, attempt, status, generation, last_failure_code \
             FROM mahfel.background_jobs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(
            |(
                job_id,
                job_type,
                version,
                payload,
                idempotency_key,
                correlation_id,
                traceparent,
                tracestate,
                created_at,
                attempt,
                status,
                generation,
                last_failure_code,
            )| JobRecord {
                envelope: JobEnvelope {
                    job_id,
                    job_type,
                    version: u16::try_from(version).unwrap_or(0),
                    created_at,
                    correlation_id,
                    traceparent,
                    tracestate,
                    idempotency_key,
                    generation: u32::try_from(generation).unwrap_or(0),
                    attempt: u16::try_from(attempt).unwrap_or(0),
                    payload,
                },
                status,
                generation,
                last_failure_code,
            },
        ))
    }

    pub async fn replay_dead(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let row: Option<(i32,)> = sqlx::query_as(
            "UPDATE mahfel.background_jobs SET status = 'queued', generation = generation + 1, \
             attempt = 1, last_failure_code = NULL, completed_at = NULL, updated_at = clock_timestamp() \
             WHERE id = $1 AND status = 'dead' RETURNING generation",
        ).bind(id).fetch_optional(&mut *tx).await?;
        let Some((generation,)) = row else {
            return Ok(false);
        };
        sqlx::query(
            "INSERT INTO mahfel.job_outbox (id, job_id, generation, attempt, available_at, created_at) \
             VALUES ($1,$2,$3,1,clock_timestamp(),clock_timestamp())",
        ).bind(Uuid::new_v4()).bind(id).bind(generation).execute(&mut *tx).await?;
        sqlx::query(
            "INSERT INTO mahfel.job_replays (job_id, generation, replayed_at) \
             VALUES ($1,$2,clock_timestamp())",
        )
        .bind(id)
        .bind(generation)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(true)
    }

    pub async fn replays(&self, id: Uuid) -> Result<Vec<JobReplay>, sqlx::Error> {
        let rows: Vec<(i32, OffsetDateTime)> = sqlx::query_as(
            "SELECT generation, replayed_at FROM mahfel.job_replays \
             WHERE job_id = $1 ORDER BY generation",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(generation, replayed_at)| JobReplay {
                generation,
                replayed_at,
            })
            .collect())
    }

    pub async fn history(&self, id: Uuid) -> Result<Vec<JobAttempt>, sqlx::Error> {
        let rows: Vec<(i32, i16, String, Option<String>, OffsetDateTime)> = sqlx::query_as(
            "SELECT generation, attempt, outcome, failure_code, finished_at \
             FROM mahfel.job_attempts WHERE job_id = $1 ORDER BY generation, attempt",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(generation, attempt, outcome, failure_code, finished_at)| JobAttempt {
                    generation,
                    attempt,
                    outcome,
                    failure_code,
                    finished_at,
                },
            )
            .collect())
    }
}
