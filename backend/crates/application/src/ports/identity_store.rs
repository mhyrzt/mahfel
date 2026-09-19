use super::StoreError;
use mahfel_domain::{AvatarUrl, DisplayName, EmailAddress, Identity, Issuer, Subject, User};
use std::future::Future;
use time::OffsetDateTime;

/// Provider-asserted profile fields carried alongside an identity.
///
/// Absent fields are not authoritative: a provider that omits a claim must not
/// erase a value already stored.
#[derive(Clone, Debug, Default)]
pub struct IdentityProfile {
    pub display_name: Option<DisplayName>,
    pub email: Option<EmailAddress>,
    pub email_verified: bool,
    pub avatar_url: Option<AvatarUrl>,
}

/// A local user together with the identity that resolved it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionedIdentity {
    pub user: User,
    pub identity: Identity,
}

/// Resolves a provider identity to a durable local user.
///
/// The store owns the atomic, conflict-safe upsert so concurrent first logins
/// for the same `(issuer, subject)` converge on a single user.
pub trait IdentityStore: Send + Sync {
    fn resolve_or_provision(
        &self,
        issuer: &Issuer,
        subject: &Subject,
        display_name: &DisplayName,
        profile: &IdentityProfile,
        now: OffsetDateTime,
    ) -> impl Future<Output = Result<ProvisionedIdentity, StoreError>> + Send;

    fn find_by_identity(
        &self,
        issuer: &Issuer,
        subject: &Subject,
    ) -> impl Future<Output = Result<Option<ProvisionedIdentity>, StoreError>> + Send;
}
