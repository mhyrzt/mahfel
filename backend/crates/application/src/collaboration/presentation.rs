use crate::ports::{
    CollaborationStore, FileStorage, FileStorageError, FileStore, NewCollaborationEvent,
    SessionStore, StoreError,
};
use mahfel_domain::{
    ActorId, CollaborationEvent, CollaborationEventKind, FileContext, FileId, FileStatus,
    MAX_FILE_BYTES, SessionId,
};
use serde_json::json;
use std::collections::HashSet;
use thiserror::Error;
use uuid::Uuid;

pub enum PresentationAction {
    Select(Vec<FileId>),
    Page(u32),
    Stop,
}

#[derive(Debug, Error)]
pub enum PresentationError {
    #[error("presentation unavailable")]
    Denied,
    #[error("invalid presentation selection")]
    Invalid,
    #[error("presentation dependency unavailable")]
    Unavailable,
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Storage(#[from] FileStorageError),
}

pub async fn change_presentation<
    C: CollaborationStore + ?Sized,
    F: FileStore + ?Sized,
    S: FileStorage + ?Sized,
    T: SessionStore + ?Sized,
>(
    collaboration: &C,
    files: &F,
    storage: &S,
    sessions: &T,
    session_id: SessionId,
    actor_id: ActorId,
    action: PresentationAction,
) -> Result<CollaborationEvent, PresentationError> {
    let access = collaboration.access(session_id, actor_id).await?;
    if !access.can_edit || !access.live {
        return Err(PresentationError::Denied);
    }
    let data = match action {
        PresentationAction::Select(file_ids) => {
            if file_ids.is_empty() || file_ids.len() > 50 {
                return Err(PresentationError::Invalid);
            }
            if file_ids.iter().collect::<HashSet<_>>().len() != file_ids.len() {
                return Err(PresentationError::Invalid);
            }
            let session = sessions
                .get(session_id)
                .await?
                .ok_or(PresentationError::Denied)?;
            let mut pdf_file = None;
            for (index, file_id) in file_ids.iter().enumerate() {
                let file = files
                    .get(*file_id)
                    .await?
                    .ok_or(PresentationError::Denied)?;
                if file.status != FileStatus::Ready
                    || !matches!(file.context, FileContext::Room(id) if id == session.room_id)
                        && !matches!(file.context, FileContext::Session(id) if id == session_id)
                {
                    return Err(PresentationError::Denied);
                }
                let is_pdf = file.verified_mime.as_deref() == Some("application/pdf");
                if !is_pdf
                    && !matches!(
                        file.verified_mime.as_deref(),
                        Some("image/png" | "image/jpeg")
                    )
                {
                    return Err(PresentationError::Invalid);
                }
                if index > 0 && (pdf_file.is_some() || is_pdf) {
                    return Err(PresentationError::Invalid);
                }
                if is_pdf {
                    pdf_file = Some(file);
                }
            }
            let pages = if let Some(file) = pdf_file {
                let bytes = storage.read(&file.object_key, MAX_FILE_BYTES).await?.bytes;
                tokio::task::spawn_blocking(move || {
                    lopdf::Document::load_mem(&bytes).map(|document| document.get_pages().len())
                })
                .await
                .map_err(|_| PresentationError::Unavailable)?
                .map_err(|_| PresentationError::Invalid)?
            } else {
                file_ids.len()
            };
            if !(1..=1000).contains(&pages) {
                return Err(PresentationError::Invalid);
            }
            json!({"action":"select","file_ids":file_ids.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "total_pages":pages,"page":1})
        }
        PresentationAction::Page(page) if page > 0 => json!({"action":"page","page":page}),
        PresentationAction::Page(_) => return Err(PresentationError::Invalid),
        PresentationAction::Stop => json!({"action":"stop"}),
    };
    collaboration
        .append(&NewCollaborationEvent {
            session_id,
            id: Uuid::new_v4(),
            actor_id,
            document_id: None,
            kind: CollaborationEventKind::Presentation,
            update: None,
            data,
        })
        .await
        .map_err(Into::into)
}
