use anyhow::{Result, anyhow};
use mahfel_application::jobs::{JobEnvelope, JobFailure, MAX_ATTEMPTS};
use mahfel_messaging::JobDelivery;
use mahfel_postgres::{JobClaim, JobStore, PostgresFileStore};
use mahfel_storage::S3FileStorage;

use super::{CLAIM_RENEW, JobMetrics, handler::handle, metrics::ActiveJob};

pub(super) async fn process_delivery(
    delivery: JobDelivery,
    store: JobStore,
    files: PostgresFileStore,
    storage: S3FileStorage,
    metrics: JobMetrics,
) -> Result<()> {
    metrics.active.inc();
    let _active = ActiveJob(metrics.active.clone());
    let job: JobEnvelope = match serde_json::from_slice(delivery.body()) {
        Ok(job) => job,
        Err(error) => {
            tracing::warn!(%error, "malformed job message sent to DLQ");
            metrics.outcomes.with_label_values(&["malformed"]).inc();
            delivery.dead_letter().await?;
            return Ok(());
        }
    };
    let claim = match store.claim(&job).await {
        Ok(claim) => claim,
        Err(error) => {
            delivery.requeue().await?;
            return Err(error.into());
        }
    };
    let token = match claim {
        JobClaim::Claimed(token) => token,
        JobClaim::Dead => {
            delivery.dead_letter().await?;
            return Ok(());
        }
        JobClaim::Busy | JobClaim::Stale | JobClaim::Terminal => {
            delivery.ack().await?;
            return Ok(());
        }
    };
    let task = handle(&job, &files, &storage);
    tokio::pin!(task);
    let mut renew = tokio::time::interval(CLAIM_RENEW);
    renew.tick().await;
    let outcome = loop {
        tokio::select! {
            result = &mut task => break result,
            _ = renew.tick() => {
                match store.renew_claim(job.job_id, token).await {
                    Ok(true) => {},
                    Ok(false) => { delivery.requeue().await?; return Err(anyhow!("job claim lost")) },
                    Err(error) => { delivery.requeue().await?; return Err(error.into()) },
                }
            }
        }
    };
    match outcome {
        Ok(()) => {
            if store.succeed(&job, token).await? {
                metrics.outcomes.with_label_values(&["succeeded"]).inc();
                tracing::info!(job_id = %job.job_id, job_type = %job.job_type,
                    generation = job.generation, attempt = job.attempt, "job succeeded");
                delivery.ack().await?;
            } else {
                delivery.requeue().await?;
            }
        }
        Err(failure) => {
            let (code, permanent) = match failure {
                JobFailure::Retryable(code) => (code, false),
                JobFailure::Permanent(code) => (code, true),
            };
            if store.fail(&job, token, code, permanent).await? {
                if permanent || job.attempt >= MAX_ATTEMPTS {
                    metrics.outcomes.with_label_values(&["dead"]).inc();
                    tracing::warn!(job_id = %job.job_id, job_type = %job.job_type,
                        generation = job.generation, attempt = job.attempt,
                        failure_code = code, "job dead-lettered");
                    delivery.dead_letter().await?;
                } else {
                    metrics.outcomes.with_label_values(&["retry"]).inc();
                    tracing::warn!(job_id = %job.job_id, job_type = %job.job_type,
                        generation = job.generation, attempt = job.attempt,
                        failure_code = code, "job retry scheduled");
                    delivery.ack().await?;
                }
            } else {
                delivery.requeue().await?;
            }
        }
    }
    Ok(())
}
