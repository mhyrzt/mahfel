use super::{JobStore, LeasedOutbox, model::OutboxRow};
use mahfel_application::jobs::JobEnvelope;
use uuid::Uuid;

impl JobStore {
    pub async fn lease_due(&self, limit: i64) -> Result<Vec<LeasedOutbox>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let rows: Vec<OutboxRow> = sqlx::query_as(
            "SELECT outbox.id, jobs.id, outbox.generation, outbox.attempt, \
                        jobs.job_type, jobs.schema_version, jobs.payload, \
                        jobs.idempotency_key, jobs.correlation_id, jobs.traceparent, \
                        jobs.tracestate, jobs.created_at \
                 FROM mahfel.job_outbox outbox \
                 JOIN mahfel.background_jobs jobs ON jobs.id = outbox.job_id \
                 WHERE outbox.published_at IS NULL AND outbox.available_at <= clock_timestamp() \
                   AND (outbox.lease_until IS NULL OR outbox.lease_until < clock_timestamp()) \
                   AND jobs.status = 'queued' AND jobs.generation = outbox.generation \
                   AND jobs.attempt = outbox.attempt \
                 ORDER BY outbox.available_at, outbox.id \
                 FOR UPDATE OF outbox SKIP LOCKED LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;
        let mut leased = Vec::with_capacity(rows.len());
        for (
            outbox_id,
            job_id,
            generation,
            attempt,
            kind,
            version,
            payload,
            key,
            correlation,
            traceparent,
            tracestate,
            created_at,
        ) in rows
        {
            let token = Uuid::new_v4();
            sqlx::query(
                "UPDATE mahfel.job_outbox SET claim_token = $2, \
                 lease_until = clock_timestamp() + INTERVAL '30 seconds' WHERE id = $1",
            )
            .bind(outbox_id)
            .bind(token)
            .execute(&mut *tx)
            .await?;
            leased.push(LeasedOutbox {
                id: outbox_id,
                token,
                envelope: JobEnvelope {
                    job_id,
                    job_type: kind,
                    version: u16::try_from(version).unwrap_or(0),
                    created_at,
                    correlation_id: correlation,
                    traceparent,
                    tracestate,
                    idempotency_key: key,
                    generation: u32::try_from(generation).unwrap_or(0),
                    attempt: u16::try_from(attempt).unwrap_or(0),
                    payload,
                },
            });
        }
        tx.commit().await?;
        Ok(leased)
    }
    pub async fn mark_published(&self, id: Uuid, token: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE mahfel.job_outbox SET published_at = clock_timestamp(), \
             claim_token = NULL, lease_until = NULL WHERE id = $1 AND claim_token = $2",
        )
        .bind(id)
        .bind(token)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn release_outbox(&self, id: Uuid, token: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE mahfel.job_outbox SET claim_token = NULL, lease_until = NULL \
             WHERE id = $1 AND claim_token = $2",
        )
        .bind(id)
        .bind(token)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn cleanup_published_outbox(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM mahfel.job_outbox WHERE published_at < clock_timestamp() - INTERVAL '7 days'",
        ).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }
}
