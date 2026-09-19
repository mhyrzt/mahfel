use crate::error::{ApplicationError, ErrorKind};
use crate::ports::{IdentityProfile, IdentityStore, ProvisionedIdentity, StoreError};
use mahfel_domain::{
    AvatarUrl, DisplayName, EmailAddress, Issuer, MAX_DISPLAY_NAME_LEN, Subject, ValidationError,
};
use thiserror::Error;
use time::OffsetDateTime;

/// Identity synced from OIDC claims (or local auth).
///
/// The transport supplies raw claim strings; this slice validates them into
/// domain value objects before touching storage.
#[derive(Debug, Clone)]
pub struct SyncIdentity {
    pub issuer: String,
    pub subject: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Error)]
pub enum SyncIdentityError {
    #[error("invalid identity claim: {0}")]
    Invalid(#[from] ValidationError),
    #[error("storage failure: {0}")]
    Store(#[from] StoreError),
}

impl ApplicationError for SyncIdentityError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Invalid(_) => ErrorKind::Unprocessable,
            Self::Store(_) => ErrorKind::Internal,
        }
    }
}

/// Resolve an authenticated identity to a durable local user.
///
/// The first login for an unseen `(issuer, subject)` provisions a user and
/// identity; later logins refresh provider profile fields. Email is never used
/// as a linking key.
pub async fn sync_identity<R>(
    identities: &R,
    command: SyncIdentity,
) -> Result<ProvisionedIdentity, SyncIdentityError>
where
    R: IdentityStore,
{
    let issuer = Issuer::new(command.issuer)?;
    let subject = Subject::new(command.subject)?;
    let email = command.email.map(EmailAddress::new).transpose()?;
    let avatar_url = command.avatar_url.map(AvatarUrl::new).transpose()?;
    let display_name = command.display_name.map(DisplayName::new).transpose()?;

    let resolved_name = fallback_display_name(display_name.as_ref(), email.as_ref(), &subject);
    let profile = IdentityProfile {
        display_name,
        email,
        email_verified: command.email_verified,
        avatar_url,
    };

    let now = OffsetDateTime::now_utc();
    Ok(identities
        .resolve_or_provision(&issuer, &subject, &resolved_name, &profile, now)
        .await?)
}

/// Derive a non-empty, length-bounded display name from the available claims.
fn fallback_display_name(
    provided: Option<&DisplayName>,
    email: Option<&EmailAddress>,
    subject: &Subject,
) -> DisplayName {
    if let Some(provided) = provided {
        return provided.clone();
    }
    let candidate = email.map_or_else(|| subject.as_str(), EmailAddress::as_str);
    let truncated: String = candidate.chars().take(MAX_DISPLAY_NAME_LEN).collect();
    DisplayName::new(truncated).expect("a non-empty bounded value is a valid display name")
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahfel_domain::MAX_SUBJECT_LEN;

    #[test]
    fn fallback_prefers_email_then_subject() {
        let subject = Subject::new("subject-1234".to_owned()).expect("subject");
        let email = EmailAddress::new("ada@example.com".to_owned()).expect("email");

        assert_eq!(
            fallback_display_name(None, Some(&email), &subject).as_str(),
            "ada@example.com"
        );
        assert_eq!(
            fallback_display_name(None, None, &subject).as_str(),
            "subject-1234"
        );
    }

    #[test]
    fn fallback_truncates_long_subjects() {
        let subject = Subject::new("s".repeat(MAX_SUBJECT_LEN)).expect("subject");
        let name = fallback_display_name(None, None, &subject);
        assert_eq!(name.as_str().chars().count(), MAX_DISPLAY_NAME_LEN);
    }
}
