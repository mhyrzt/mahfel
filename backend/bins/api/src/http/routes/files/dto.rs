use mahfel_domain::{ActorId, ChatMessageId, FileContext, FileObject, RoomId, SessionId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
/// The resource that owns or contains a file.
pub(crate) enum FileContextKind {
    /// A room's shared files.
    Room,
    /// A session's shared files.
    Session,
    /// A file attached to a chat message.
    ChatMessage,
    /// The current actor's avatar.
    Avatar,
    /// A recording associated with a session.
    Recording,
}

#[derive(Debug, Deserialize, ToSchema)]
/// Selects a file context and page for listing ready files.
pub(crate) struct FileContextInput {
    /// Context type, such as `room`, `session`, or `chat_message`.
    pub(crate) context_kind: FileContextKind,
    /// Identifier of the selected context.
    pub(crate) context_id: Uuid,
    /// Maximum number of files to return, from 1 to 100.
    pub(crate) limit: Option<u32>,
    /// Number of ready files to skip.
    pub(crate) offset: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// Metadata and context used to authorize a direct upload.
pub(crate) struct InitiateUploadRequest {
    /// Context type that will own the uploaded file.
    pub(crate) context_kind: FileContextKind,
    /// Identifier of the selected context.
    pub(crate) context_id: Uuid,
    /// Display filename; the server stores bytes under a generated key.
    pub(crate) file_name: String,
    /// Declared MIME type. The server verifies content before making it ready.
    pub(crate) mime: String,
    /// Expected object size in bytes, limited to 50 MiB.
    pub(crate) size_bytes: u64,
    /// Base64-encoded SHA-256 checksum required by the signed upload.
    pub(crate) checksum_sha256: String,
}

#[derive(Debug, Serialize, ToSchema)]
/// File metadata returned by the API. Object bytes stay in S3-compatible storage.
pub(crate) struct FileResponse {
    id: Uuid,
    owner_id: Uuid,
    context_kind: FileContextKind,
    context_id: Uuid,
    status: String,
    file_name: String,
    mime: String,
    size_bytes: u64,
    checksum_sha256: String,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
/// A short-lived storage URL and the headers required to use it.
pub(crate) struct SignedFileResponse {
    pub(super) file: FileResponse,
    pub(super) url: String,
    pub(super) headers: Vec<SignedHeader>,
    pub(super) expires_in_seconds: u64,
}

#[derive(Debug, Serialize, ToSchema)]
/// A header that must accompany a signed storage request.
pub(crate) struct SignedHeader {
    pub(super) name: String,
    pub(super) value: String,
}

#[derive(Debug, Serialize, ToSchema)]
/// Ready files visible in a context.
pub(crate) struct FileListResponse {
    pub(super) files: Vec<FileResponse>,
}

impl From<FileObject> for FileResponse {
    fn from(file: FileObject) -> Self {
        let (context_kind, context_id) = match file.context {
            FileContext::Room(id) => (FileContextKind::Room, id.as_uuid()),
            FileContext::Session(id) => (FileContextKind::Session, id.as_uuid()),
            FileContext::ChatMessage(id) => (FileContextKind::ChatMessage, id.as_uuid()),
            FileContext::Avatar(id) => (FileContextKind::Avatar, id.as_uuid()),
            FileContext::Recording(id) => (FileContextKind::Recording, id.as_uuid()),
        };
        Self {
            id: file.id.as_uuid(),
            owner_id: file.owner.as_uuid(),
            context_kind,
            context_id,
            status: file.status.as_str().to_owned(),
            file_name: file.file_name,
            mime: file.verified_mime.unwrap_or(file.declared_mime),
            size_bytes: file.size_bytes,
            checksum_sha256: file.checksum_sha256,
            created_at: file.created_at,
            updated_at: file.updated_at,
        }
    }
}

pub(super) fn context(kind: FileContextKind, id: Uuid) -> FileContext {
    match kind {
        FileContextKind::Room => FileContext::Room(RoomId::from_uuid(id)),
        FileContextKind::Session => FileContext::Session(SessionId::from_uuid(id)),
        FileContextKind::ChatMessage => FileContext::ChatMessage(ChatMessageId::from_uuid(id)),
        FileContextKind::Avatar => FileContext::Avatar(ActorId::from_uuid(id)),
        FileContextKind::Recording => FileContext::Recording(SessionId::from_uuid(id)),
    }
}
