use anyhow::Result;
use mahfel_postgres::PostgresRecordingStore;
use std::{path::Path, time::Duration};
use uuid::Uuid;

pub(crate) async fn cleanup_scratch(root: &Path, store: &PostgresRecordingStore) -> Result<()> {
    let mut entries = match tokio::fs::read_dir(root).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.len() != 73 || !name.is_ascii() || name.as_bytes()[36] != b'-' {
            continue;
        }
        let (Ok(_), Ok(token)) = (Uuid::parse_str(&name[..36]), Uuid::parse_str(&name[37..]))
        else {
            continue;
        };
        let age = entry
            .metadata()
            .await?
            .modified()?
            .elapsed()
            .unwrap_or_default();
        if age >= Duration::from_secs(86400) && !store.export_claim_active(token).await? {
            tokio::fs::remove_dir_all(entry.path()).await?;
        }
    }
    Ok(())
}
