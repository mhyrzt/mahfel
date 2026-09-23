use crate::ids::entity_id;
use crate::{ActorId, ChatMessageId, RoomId, SessionId};
use base64::Engine;
use time::OffsetDateTime;

entity_id!(FileId);

pub const MAX_FILE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_OWNER_FILE_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileContext {
    Room(RoomId),
    Session(SessionId),
    ChatMessage(ChatMessageId),
    Avatar(ActorId),
    Recording(SessionId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileStatus {
    Pending,
    Quarantined,
    Ready,
    Rejected,
    Deleting,
}

impl FileStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Quarantined => "quarantined",
            Self::Ready => "ready",
            Self::Rejected => "rejected",
            Self::Deleting => "deleting",
        }
    }

    #[must_use]
    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "quarantined" => Some(Self::Quarantined),
            "ready" => Some(Self::Ready),
            "rejected" => Some(Self::Rejected),
            "deleting" => Some(Self::Deleting),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileObject {
    pub id: FileId,
    pub object_key: String,
    pub owner: ActorId,
    pub context: FileContext,
    pub status: FileStatus,
    pub file_name: String,
    pub declared_mime: String,
    pub verified_mime: Option<String>,
    pub size_bytes: u64,
    pub checksum_sha256: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("file name must be 1 to 255 characters")]
    InvalidName,
    #[error("file size must be within the allowed limit")]
    InvalidSize,
    #[error("invalid SHA-256 checksum")]
    InvalidChecksum,
    #[error("unsupported file type")]
    UnsupportedType,
}

impl FileObject {
    pub fn pending(
        owner: ActorId,
        context: FileContext,
        file_name: String,
        declared_mime: String,
        size_bytes: u64,
        checksum_sha256: String,
        now: OffsetDateTime,
    ) -> Result<Self, FileError> {
        if file_name.is_empty() || file_name.len() > 255 || file_name.chars().any(char::is_control)
        {
            return Err(FileError::InvalidName);
        }
        if size_bytes == 0 || size_bytes > MAX_FILE_BYTES {
            return Err(FileError::InvalidSize);
        }
        if !matches!(
            declared_mime.as_str(),
            "application/pdf" | "image/png" | "image/jpeg"
        ) {
            return Err(FileError::UnsupportedType);
        }
        let checksum = base64::engine::general_purpose::STANDARD
            .decode(&checksum_sha256)
            .map_err(|_| FileError::InvalidChecksum)?;
        if checksum.len() != 32
            || base64::engine::general_purpose::STANDARD.encode(&checksum) != checksum_sha256
        {
            return Err(FileError::InvalidChecksum);
        }
        let id = FileId::new();
        Ok(Self {
            id,
            object_key: format!("uploads/{id}"),
            owner,
            context,
            status: FileStatus::Pending,
            file_name,
            declared_mime,
            verified_mime: None,
            size_bytes,
            checksum_sha256,
            created_at: now,
            updated_at: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{FileContext, FileError, FileObject};
    use crate::{ActorId, RoomId};
    use time::OffsetDateTime;

    #[test]
    fn keys_are_random_and_metadata_is_bounded() {
        let make = || {
            FileObject::pending(
                ActorId::new(),
                FileContext::Room(RoomId::new()),
                "deck.pdf".into(),
                "application/pdf".into(),
                1024,
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
                OffsetDateTime::UNIX_EPOCH,
            )
        };
        let first = make().expect("valid file");
        let second = make().expect("valid file");
        assert_ne!(first.object_key, second.object_key);
        assert!(first.object_key.starts_with("uploads/"));
        assert!(matches!(
            FileObject::pending(
                first.owner,
                first.context,
                "x".into(),
                "application/pdf".into(),
                0,
                first.checksum_sha256.clone(),
                OffsetDateTime::UNIX_EPOCH
            ),
            Err(FileError::InvalidSize)
        ));
        assert!(matches!(
            FileObject::pending(
                first.owner,
                first.context,
                "x".into(),
                "application/pdf".into(),
                1,
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB=".into(),
                OffsetDateTime::UNIX_EPOCH
            ),
            Err(FileError::InvalidChecksum)
        ));
    }
}
