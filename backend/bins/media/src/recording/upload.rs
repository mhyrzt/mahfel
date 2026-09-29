use crate::recording::{Recordings, spool::paths};
use mahfel_application::recording_artifacts::FragmentReceipt;
use std::{sync::atomic::Ordering, time::Duration};
impl Recordings {
    pub fn spawn_uploader(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) {
        let owner = self.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown.borrow() {
                    break;
                }
                if let Err(error) = owner.upload_pending().await {
                    owner.metrics.uploads.with_label_values(&["retry"]).inc();
                    owner.upload_failed.store(true, Ordering::Release);
                    tracing::warn!(%error, "recording upload retry pending");
                } else {
                    owner.metrics.uploads.with_label_values(&["success"]).inc();
                    owner.upload_failed.store(false, Ordering::Release);
                }
                tokio::select! { _ = tokio::time::sleep(Duration::from_secs(1)) => {}, _ = shutdown.changed() => break }
            }
        });
    }

    pub(super) async fn upload_pending(&self) -> anyhow::Result<()> {
        let root = self.settings.spool_directory.clone();
        let journals = tokio::task::spawn_blocking(move || paths(root, Some("receipt"))).await??;
        let mut tasks = tokio::task::JoinSet::new();
        let mut failed = false;
        for journal in journals {
            while tasks.len() >= self.settings.upload_concurrency {
                if let Some(result) = tasks.join_next().await
                    && !matches!(result, Ok(Ok(())))
                {
                    failed = true;
                }
            }
            let storage = self.storage.clone();
            tasks.spawn(async move {
                let receipt: FragmentReceipt =
                    serde_json::from_slice(&tokio::fs::read(&journal).await?)?;
                let fragment = journal.with_extension(&receipt.container);
                storage.put_fragment(&receipt.object_key, &fragment).await?;
                let key = format!(
                    "recordings/{}/capture/{}/{}/receipts/{}-{}-{}.json",
                    receipt.recording_id,
                    receipt.generation,
                    receipt.boot_id,
                    receipt.producer_id,
                    fragment
                        .parent()
                        .and_then(|path| path.file_name())
                        .and_then(|value| value.to_str())
                        .ok_or_else(|| anyhow::anyhow!("invalid fragment instance"))?,
                    fragment
                        .file_stem()
                        .and_then(|value| value.to_str())
                        .ok_or_else(|| anyhow::anyhow!("invalid fragment filename"))?
                );
                storage.put(&key, serde_json::to_vec(&receipt)?).await?;
                // Keep the journal until both objects are durable; retries overwrite identical bytes.
                tokio::fs::remove_file(&journal).await?;
                tokio::fs::remove_file(&fragment).await?;
                Ok::<_, anyhow::Error>(())
            });
        }
        while let Some(result) = tasks.join_next().await {
            if !matches!(result, Ok(Ok(()))) {
                failed = true;
            }
        }
        anyhow::ensure!(!failed, "one or more fragment uploads remain pending");
        Ok(())
    }
}
