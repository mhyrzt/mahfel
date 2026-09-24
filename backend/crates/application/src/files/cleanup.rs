use crate::ports::{FileStorage, FileStore};
use mahfel_domain::{FileId, FileStatus};
use time::{Duration, OffsetDateTime};

pub async fn cleanup_expired<F: FileStore + ?Sized, S: FileStorage + ?Sized>(
    files: &F,
    storage: &S,
) -> Result<u32, crate::files::FileOperationError> {
    let cutoff = OffsetDateTime::now_utc() - Duration::minutes(30);
    let candidates = files.pending_before(cutoff, 100).await?;
    let mut removed = 0;
    for file in candidates {
        if file.status != FileStatus::Deleting
            && !files
                .transition(
                    file.id,
                    file.status,
                    FileStatus::Deleting,
                    None,
                    OffsetDateTime::now_utc(),
                )
                .await?
        {
            continue;
        }
        if let Err(error) = cleanup_file(files, storage, file.id).await {
            tracing::warn!(file_id = %file.id, %error, "expired file object deletion failed");
            continue;
        }
        removed += 1;
    }
    Ok(removed)
}

/// Deleting a missing object or metadata row is successful, so redelivery is safe.
pub async fn cleanup_file<F: FileStore + ?Sized, S: FileStorage + ?Sized>(
    files: &F,
    storage: &S,
    id: FileId,
) -> Result<(), crate::files::FileOperationError> {
    let Some(file) = files.get(id).await? else {
        return Ok(());
    };
    if file.status != FileStatus::Deleting {
        return Err(crate::files::FileOperationError::NotReady);
    }
    storage.delete(&file.object_key).await?;
    storage.delete(&format!("uploads/{}", file.id)).await?;
    storage.delete(&format!("files/{}", file.id)).await?;
    files.remove(file.id).await?;
    Ok(())
}
