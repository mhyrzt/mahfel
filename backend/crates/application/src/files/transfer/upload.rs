use crate::files::FileOperationError;
use crate::files::access::{FileAccess, require_access};
use crate::files::validation::verify;
use crate::ports::{FileStorage, FileStore, PresignedFileRequest, RoomStore, StoredFile};
use mahfel_domain::{ActorId, FileContext, FileId, FileObject, FileStatus, MAX_FILE_BYTES};
use std::{sync::OnceLock, time::Duration};
use time::OffsetDateTime;
use tokio::sync::Semaphore;

const UPLOAD_EXPIRY: Duration = Duration::from_secs(300);

pub struct InitiateUpload {
    pub actor: ActorId,
    pub context: FileContext,
    pub file_name: String,
    pub mime: String,
    pub size_bytes: u64,
    pub checksum_sha256: String,
}

pub struct CompleteUpload {
    pub actor: ActorId,
    pub id: FileId,
}

pub async fn initiate_upload<
    F: FileStore + ?Sized,
    R: RoomStore + ?Sized,
    S: FileStorage + ?Sized,
>(
    files: &F,
    rooms: &R,
    storage: &S,
    input: InitiateUpload,
) -> Result<(FileObject, PresignedFileRequest), FileOperationError> {
    require_access(files, rooms, input.context, input.actor, FileAccess::Upload).await?;
    let file = FileObject::pending(
        input.actor,
        input.context,
        input.file_name,
        input.mime,
        input.size_bytes,
        input.checksum_sha256,
        OffsetDateTime::now_utc(),
    )?;
    files.insert(&file).await?;
    let signed = storage
        .presign_upload(
            &file.object_key,
            &file.declared_mime,
            file.size_bytes,
            &file.checksum_sha256,
            UPLOAD_EXPIRY,
        )
        .await?;
    Ok((file, signed))
}

pub async fn complete_upload<F: FileStore + ?Sized, S: FileStorage + ?Sized>(
    files: &F,
    storage: &S,
    input: CompleteUpload,
) -> Result<FileObject, FileOperationError> {
    let file = files
        .get(input.id)
        .await?
        .ok_or(FileOperationError::NotFound)?;
    if file.owner != input.actor {
        return Err(FileOperationError::NotFound);
    }
    if file.status == FileStatus::Ready {
        return Ok(file);
    }
    if file.status == FileStatus::Pending {
        if !files
            .transition(
                file.id,
                FileStatus::Pending,
                FileStatus::Quarantined,
                None,
                OffsetDateTime::now_utc(),
            )
            .await?
        {
            return Err(FileOperationError::NotReady);
        }
    } else if file.status != FileStatus::Quarantined {
        return Err(FileOperationError::NotReady);
    }
    let stored = storage.read(&file.object_key, MAX_FILE_BYTES).await?;
    let now = OffsetDateTime::now_utc();
    let Some(mime) = verify_upload(file.clone(), stored).await? else {
        files
            .transition(
                file.id,
                FileStatus::Quarantined,
                FileStatus::Rejected,
                None,
                now,
            )
            .await?;
        return Err(FileOperationError::Rejected);
    };
    let final_key = format!("files/{}", file.id);
    storage.promote(&file.object_key, &final_key).await?;
    let published = storage.read(&final_key, MAX_FILE_BYTES).await?;
    if verify_upload(file.clone(), published).await?.is_none() {
        files
            .transition(
                file.id,
                FileStatus::Quarantined,
                FileStatus::Rejected,
                None,
                now,
            )
            .await?;
        return Err(FileOperationError::Rejected);
    }
    if !files.publish(file.id, &final_key, mime, now).await? {
        let current = files.get(file.id).await?;
        if let Some(ready) = current
            .as_ref()
            .filter(|current| current.status == FileStatus::Ready)
        {
            return Ok(ready.clone());
        }
        storage.delete(&final_key).await?;
        return Err(FileOperationError::NotReady);
    }
    if let Err(error) = storage.delete(&file.object_key).await {
        tracing::warn!(file_id = %file.id, %error, "verified upload staging object cleanup failed");
    }
    files
        .get(file.id)
        .await?
        .ok_or(FileOperationError::NotFound)
}

async fn verify_upload(
    file: FileObject,
    stored: StoredFile,
) -> Result<Option<&'static str>, FileOperationError> {
    static VERIFICATION_SLOTS: OnceLock<Semaphore> = OnceLock::new();
    let permit = VERIFICATION_SLOTS
        .get_or_init(|| Semaphore::new(4))
        .acquire()
        .await
        .map_err(|_| FileOperationError::VerificationUnavailable)?;
    let result = tokio::task::spawn_blocking(move || verify(&file, &stored))
        .await
        .map_err(|_| FileOperationError::VerificationUnavailable);
    drop(permit);
    result
}
