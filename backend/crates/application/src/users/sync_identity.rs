use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{StoreError, UserStore};
use mahfel_domain::{DisplayName, User, ValidationError};
use thiserror::Error;
use time::OffsetDateTime;

/// Identity synced from OIDC claims (or local auth).
/// Vertical slice that will back `SyncAuthenticatedUser` use case.
#[derive(Debug, Clone)]
pub struct SyncIdentity {
    pub subject: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Error)]
pub enum SyncIdentityError {
    #[error("invalid display name: {0}")]
    InvalidDisplayName(#[from] ValidationError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for SyncIdentityError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::InvalidDisplayName(_) => ErrorKind::Unprocessable,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

pub async fn sync_identity<R>(_users: &R, _command: SyncIdentity) -> Result<User, SyncIdentityError>
where
    R: UserStore,
{
    // Placeholder slice — actual upsert (subject → User) will be added
    // once `mahfel-postgres` exposes a subject-indexed store and the
    // `users` table gains the linking columns planned for M2.
    // For now we create an ephemeral domain user to exercise the
    // validation path and keep the port boundary honest.
    let name = _command
        .display_name
        .or(_command.email)
        .unwrap_or_else(|| _command.subject.clone());
    let display_name = DisplayName::new(name)?;
    let now = OffsetDateTime::now_utc();
    Ok(User::create(display_name, now))
}
