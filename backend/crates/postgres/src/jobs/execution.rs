use super::{JobClaim, JobStore};
use mahfel_application::jobs::{JobEnvelope, MAX_ATTEMPTS, retry_delay_seconds};
use uuid::Uuid;

impl JobStore {
    pub fn recordings(&self) -> crate::PostgresRecordingStore {
        crate::PostgresRecordingStore::new(self.pool.clone())
    }
    pub async fn claim(&self, envelope: &JobEnvelope) -> Result<JobClaim, sqlx::Error> {
        let token = Uuid::new_v4();
        let updated = sqlx::query_scalar::<_, Uuid>(
            "UPDATE mahfel.background_jobs SET status = 'running', claim_token = $4, \
             lease_until = clock_timestamp() + INTERVAL '2 minutes', updated_at = clock_timestamp() \
             WHERE id = $1 AND generation = $2 AND attempt = $3 \
               AND (status = 'queued' OR (status = 'running' AND lease_until < clock_timestamp())) \
             RETURNING id",
        )
        .bind(envelope.job_id)
        .bind(i32::try_from(envelope.generation).unwrap_or(i32::MAX))
        .bind(i16::try_from(envelope.attempt).unwrap_or(i16::MAX))
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;
        if updated.is_some() {
            return Ok(JobClaim::Claimed(token));
        }
        let state: Option<(String, i32, i16)> = sqlx::query_as(
            "SELECT status, generation, attempt FROM mahfel.background_jobs WHERE id = $1",
        )
        .bind(envelope.job_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(match state {
            Some((status, generation, attempt))
                if status == "running"
                    && generation == i32::try_from(envelope.generation).unwrap_or(0)
                    && attempt == i16::try_from(envelope.attempt).unwrap_or(0) =>
            {
                JobClaim::Busy
            }
            Some((status, _, _)) if status == "dead" => JobClaim::Dead,
            Some((status, _, _)) if status == "succeeded" || status == "cancelled" => {
                JobClaim::Terminal
            }
            _ => JobClaim::Stale,
        })
    }

    pub async fn renew_claim(&self, id: Uuid, token: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE mahfel.background_jobs SET lease_until = clock_timestamp() + INTERVAL '2 minutes' \
             WHERE id = $1 AND claim_token = $2 AND status = 'running'",
        )
        .bind(id)
        .bind(token)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn succeed(&self, job: &JobEnvelope, token: Uuid) -> Result<bool, sqlx::Error> {
        self.finish(job, token, None).await
    }

    pub async fn fail(
        &self,
        job: &JobEnvelope,
        token: Uuid,
        code: &'static str,
        permanent: bool,
    ) -> Result<bool, sqlx::Error> {
        self.finish(job, token, Some((code, permanent))).await
    }

    async fn finish(
        &self,
        job: &JobEnvelope,
        token: Uuid,
        failure: Option<(&'static str, bool)>,
    ) -> Result<bool, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let status = if let Some((_, permanent)) = failure {
            if permanent || job.attempt >= MAX_ATTEMPTS {
                "dead"
            } else {
                "queued"
            }
        } else {
            "succeeded"
        };
        let updated = sqlx::query_scalar::<_, Uuid>(
            "UPDATE mahfel.background_jobs SET status = $3, \
             attempt = CASE WHEN $3 = 'queued' THEN attempt + 1 ELSE attempt END, \
             claim_token = NULL, lease_until = NULL, last_failure_code = $4, \
             completed_at = CASE WHEN $3 IN ('succeeded','dead') THEN clock_timestamp() ELSE NULL END, \
             updated_at = clock_timestamp() \
             WHERE id = $1 AND claim_token = $2 AND status = 'running' \
             AND generation = $5 AND attempt = $6 RETURNING id",
        )
        .bind(job.job_id)
        .bind(token)
        .bind(status)
        .bind(failure.map(|(code, _)| code))
        .bind(i32::try_from(job.generation).unwrap_or(i32::MAX))
        .bind(i16::try_from(job.attempt).unwrap_or(i16::MAX))
        .fetch_optional(&mut *tx)
        .await?;
        if updated.is_none() {
            return Ok(false);
        }
        let outcome = match status {
            "queued" => "retry",
            "dead" => "dead",
            _ => "succeeded",
        };
        sqlx::query(
            "INSERT INTO mahfel.job_attempts \
             (job_id, generation, attempt, outcome, failure_code, finished_at) \
             VALUES ($1,$5,$2,$3,$4,clock_timestamp()) \
             ON CONFLICT (job_id, generation, attempt) DO NOTHING",
        )
        .bind(job.job_id)
        .bind(i16::try_from(job.attempt).unwrap_or(i16::MAX))
        .bind(outcome)
        .bind(failure.map(|(code, _)| code))
        .bind(i32::try_from(job.generation).unwrap_or(i32::MAX))
        .execute(&mut *tx)
        .await?;
        if status == "queued" {
            let next_attempt = job.attempt + 1;
            let delay = retry_delay_seconds(next_attempt).unwrap_or(600);
            sqlx::query(
                "INSERT INTO mahfel.job_outbox \
                 (id, job_id, generation, attempt, available_at, created_at) \
                 VALUES ($1,$2,$5,$3,clock_timestamp() + ($4 * INTERVAL '1 second'),clock_timestamp()) \
                 ON CONFLICT (job_id, generation, attempt) DO NOTHING",
            )
            .bind(Uuid::new_v4())
            .bind(job.job_id)
            .bind(i16::try_from(next_attempt).unwrap_or(i16::MAX))
            .bind(i64::try_from(delay).unwrap_or(600))
            .bind(i32::try_from(job.generation).unwrap_or(i32::MAX))
            .execute(&mut *tx)
            .await?;
        }
        if status == "dead"
            && job.job_type == mahfel_application::jobs::RECORDING_FINALIZE_KIND
            && let Some(id) = job.payload["recording_id"]
                .as_str()
                .and_then(|id| Uuid::parse_str(id).ok())
        {
            sqlx::query("UPDATE mahfel.recordings SET status = 'failed', warning = 'finalization_failed' WHERE id = $1 AND status = 'finalizing'")
                    .bind(id).execute(&mut *tx).await?;
        }
        if job.job_type == mahfel_application::jobs::RECORDING_EXPORT_KIND {
            sqlx::query("UPDATE mahfel.recording_exports SET status = CASE WHEN $2 = 'dead' THEN 'failed' WHEN $2 = 'queued' THEN 'queued' ELSE status END, failure_code = $3, updated_at = clock_timestamp() WHERE job_id = $1 AND status <> 'ready'")
                .bind(job.job_id).bind(status).bind(failure.map(|(code, _)| code)).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(true)
    }
    pub async fn recover_expired(&self) -> Result<u64, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let expired: Vec<(Uuid, i32, i16)> = sqlx::query_as(
            "UPDATE mahfel.background_jobs SET status = 'queued', claim_token = NULL, \
             lease_until = NULL, updated_at = clock_timestamp() \
             WHERE status = 'running' AND lease_until < clock_timestamp() \
             RETURNING id, generation, attempt",
        )
        .fetch_all(&mut *tx)
        .await?;
        for (id, generation, attempt) in &expired {
            sqlx::query(
                "UPDATE mahfel.job_outbox SET published_at = NULL, available_at = clock_timestamp(), \
                 claim_token = NULL, lease_until = NULL \
                 WHERE job_id = $1 AND generation = $2 AND attempt = $3",
            )
            .bind(id)
            .bind(generation)
            .bind(attempt)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(expired.len() as u64)
    }
}
