use crate::files::FileOperationError;
use crate::files::access::{FileAccess, require_access};
use crate::ports::{FileStorage, FileStore, PresignedFileRequest, RoomStore};
use mahfel_domain::{ActorId, FileContext, FileId, FileObject, FileStatus, SessionId};
use std::time::Duration;
use time::OffsetDateTime;

const DOWNLOAD_EXPIRY: Duration = Duration::from_secs(60);

pub struct DownloadFile {
    pub actor: ActorId,
    pub id: FileId,
}

pub struct DeleteFile {
    pub actor: ActorId,
    pub id: FileId,
}

pub struct ListFiles {
    pub actor: ActorId,
    pub context: FileContext,
    pub limit: u32,
    pub offset: u64,
}

pub struct LinkRoomFile {
    pub actor: ActorId,
    pub file_id: FileId,
    pub session_id: SessionId,
}

pub async fn link_room_file<F: FileStore + ?Sized, R: RoomStore + ?Sized>(
    files: &F,
    rooms: &R,
    input: LinkRoomFile,
) -> Result<FileObject, FileOperationError> {
    let file = files
        .get(input.file_id)
        .await?
        .ok_or(FileOperationError::NotFound)?;
    let FileContext::Room(source_room) = file.context else {
        return Err(FileOperationError::NotFound);
    };
    if file.status != FileStatus::Ready {
        return Err(FileOperationError::NotReady);
    }
    let target = FileContext::Session(input.session_id);
    let target_room = files
        .context_room(target)
        .await?
        .ok_or(FileOperationError::NotFound)?;
    if source_room != target_room {
        return Err(FileOperationError::NotFound);
    }
    require_access(files, rooms, file.context, input.actor, FileAccess::Read).await?;
    require_access(files, rooms, target, input.actor, FileAccess::Upload).await?;
    if !files
        .link_room_file_to_session(file.id, input.session_id)
        .await?
    {
        return Err(FileOperationError::NotFound);
    }
    Ok(file)
}

pub async fn download_file<
    F: FileStore + ?Sized,
    R: RoomStore + ?Sized,
    S: FileStorage + ?Sized,
>(
    files: &F,
    rooms: &R,
    storage: &S,
    input: DownloadFile,
) -> Result<(FileObject, PresignedFileRequest), FileOperationError> {
    let file = files
        .get(input.id)
        .await?
        .ok_or(FileOperationError::NotFound)?;
    require_access(files, rooms, file.context, input.actor, FileAccess::Read).await?;
    if file.status != FileStatus::Ready {
        return Err(FileOperationError::NotFound);
    }
    let signed = storage
        .presign_download(&file.object_key, &file.file_name, DOWNLOAD_EXPIRY)
        .await?;
    files
        .record_download(file.id, input.actor, OffsetDateTime::now_utc())
        .await?;
    tracing::info!(file_id = %file.id, actor_id = %input.actor, "file download authorized");
    Ok((file, signed))
}

pub async fn list_files<F: FileStore + ?Sized, R: RoomStore + ?Sized>(
    files: &F,
    rooms: &R,
    input: ListFiles,
) -> Result<Vec<FileObject>, FileOperationError> {
    require_access(files, rooms, input.context, input.actor, FileAccess::Read).await?;
    Ok(files
        .list(input.context, input.limit.clamp(1, 100), input.offset)
        .await?)
}

pub async fn delete_file<F: FileStore + ?Sized, R: RoomStore + ?Sized, S: FileStorage + ?Sized>(
    files: &F,
    rooms: &R,
    storage: &S,
    input: DeleteFile,
) -> Result<(), FileOperationError> {
    let file = files
        .get(input.id)
        .await?
        .ok_or(FileOperationError::NotFound)?;
    if file.owner != input.actor {
        require_access(files, rooms, file.context, input.actor, FileAccess::Delete).await?;
    }
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
        return Err(FileOperationError::NotReady);
    }
    storage.delete(&file.object_key).await?;
    storage.delete(&format!("uploads/{}", file.id)).await?;
    storage.delete(&format!("files/{}", file.id)).await?;
    files.remove(file.id).await?;
    Ok(())
}
