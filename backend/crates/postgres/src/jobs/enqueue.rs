use mahfel_application::jobs::JobEnvelope;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// Call inside the same transaction as the originating business mutation.
pub async fn insert_job_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    job: &JobEnvelope,
) -> Result<(), sqlx::Error> {
    let inserted = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO mahfel.background_jobs \
         (id, job_type, schema_version, payload, idempotency_key, correlation_id, traceparent, tracestate, status, \
          generation, attempt, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'queued',1,1,$9,$9) \
         ON CONFLICT (job_type, idempotency_key) DO NOTHING RETURNING id",
    )
    .bind(job.job_id)
    .bind(&job.job_type)
    .bind(i16::try_from(job.version).unwrap_or(i16::MAX))
    .bind(&job.payload)
    .bind(&job.idempotency_key)
    .bind(&job.correlation_id)
    .bind(&job.traceparent)
    .bind(&job.tracestate)
    .bind(job.created_at)
    .fetch_optional(&mut **tx)
    .await?;
    if let Some(job_id) = inserted {
        sqlx::query(
            "INSERT INTO mahfel.job_outbox \
             (id, job_id, generation, attempt, available_at, created_at) \
             VALUES ($1,$2,1,1,$3,$3)",
        )
        .bind(Uuid::new_v4())
        .bind(job_id)
        .bind(job.created_at)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}
