use crate::ports::{FileStorage, FileStore};
use mahfel_domain::FileStatus;
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
        let deleted = async {
            storage.delete(&file.object_key).await?;
            storage.delete(&format!("uploads/{}", file.id)).await?;
            storage.delete(&format!("files/{}", file.id)).await
        }
        .await;
        if let Err(error) = deleted {
            tracing::warn!(file_id = %file.id, %error, "expired file object deletion failed");
            continue;
        }
        files.remove(file.id).await?;
        removed += 1;
    }
    Ok(removed)
}
