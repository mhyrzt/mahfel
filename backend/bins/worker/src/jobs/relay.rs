use super::{JobMetrics, OUTBOX_POLL};
use anyhow::{Context, Result};
use mahfel_messaging::ConfirmedPublisher;
use mahfel_postgres::JobStore;

pub(super) async fn relay_loop(
    store: JobStore,
    publisher: ConfirmedPublisher,
    metrics: JobMetrics,
) -> Result<()> {
    let mut poll = tokio::time::interval(OUTBOX_POLL);
    let mut ticks = 0_u64;
    loop {
        poll.tick().await;
        let (pending, oldest) = store.pending_stats().await.context("read outbox lag")?;
        metrics.pending.set(pending);
        metrics.oldest.set(oldest.max(0.0));
        for row in store.lease_due(20).await.context("lease outbox")? {
            match publisher.publish(&row.envelope).await {
                Ok(()) => {
                    if store
                        .mark_published(row.id, row.token)
                        .await
                        .context("record publish confirm")?
                    {
                        metrics.publishes.with_label_values(&["confirmed"]).inc();
                    } else {
                        metrics
                            .publishes
                            .with_label_values(&["stale_confirm"])
                            .inc();
                        tracing::warn!(outbox_id = %row.id, "publish confirmation lost its outbox lease");
                    }
                }
                Err(error) => {
                    metrics.publishes.with_label_values(&["failed"]).inc();
                    store
                        .release_outbox(row.id, row.token)
                        .await
                        .context("release failed publish")?;
                    return Err(error.into());
                }
            }
        }
        ticks += 1;
        if ticks.is_multiple_of(300) {
            store
                .cleanup_published_outbox()
                .await
                .context("clean confirmed outbox")?;
        }
    }
}
