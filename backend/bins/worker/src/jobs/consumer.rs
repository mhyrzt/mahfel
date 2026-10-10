use anyhow::{Result, anyhow};
use mahfel_config::WorkerSettings;
use mahfel_messaging::JobConsumer;
use mahfel_postgres::{JobStore, PostgresFileStore};
use mahfel_storage::S3FileStorage;
use std::time::Duration;
use tokio::{sync::watch, task::JoinSet};

use super::{JobMetrics, processor::process_delivery};

pub(super) async fn consume_loop(
    mut consumer: JobConsumer,
    store: JobStore,
    files: PostgresFileStore,
    storage: S3FileStorage,
    metrics: JobMetrics,
    mut shutdown: watch::Receiver<bool>,
    settings: WorkerSettings,
) -> Result<()> {
    let mut active = JoinSet::new();
    loop {
        tokio::select! {
            _ = shutdown.changed() => break,
            result = active.join_next(), if !active.is_empty() => {
                if let Some(result) = result {
                    match result {
                        Ok(Ok(())) => {},
                        Ok(Err(error)) => tracing::warn!(%error, "job delivery failed; other deliveries remain active"),
                        Err(error) => tracing::warn!(%error, "job task failed; claim recovery will permit redelivery"),
                    }
                }
            }
            delivery = consumer.next(), if active.len() < settings.concurrency => {
                let Some(delivery) = delivery else { return Err(anyhow!("consumer stream closed")) };
                let delivery = delivery?;
                active.spawn(process_delivery(delivery, settings.exports.clone(), store.clone(), files.clone(), storage.clone(), metrics.clone()));
            }
        }
    }
    consumer.stop().await?;
    let drained =
        tokio::time::timeout(Duration::from_secs(settings.drain_timeout_seconds), async {
            while let Some(result) = active.join_next().await {
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => tracing::warn!(%error, "job delivery failed while draining"),
                    Err(error) => tracing::warn!(%error, "job task failed while draining"),
                }
            }
            Ok::<(), anyhow::Error>(())
        })
        .await;
    match drained {
        Ok(result) => result,
        Err(_) => {
            active.abort_all();
            while active.join_next().await.is_some() {}
            Ok(())
        }
    }
}
