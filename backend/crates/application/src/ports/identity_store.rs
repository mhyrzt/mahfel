use super::{BoxFuture, StoreError};
use mahfel_domain::{AvatarUrl, DisplayName, EmailAddress, Identity, Issuer, Subject, User};
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
/// for the same `(issuer, subject)` converge on a single user. Methods return
/// [`BoxFuture`] so the port is dyn-compatible.
pub trait IdentityStore: Send + Sync {
    fn resolve_or_provision<'a>(
        &'a self,
        issuer: &'a Issuer,
        subject: &'a Subject,
        display_name: &'a DisplayName,
        profile: &'a IdentityProfile,
        now: OffsetDateTime,
    ) -> BoxFuture<'a, Result<ProvisionedIdentity, StoreError>>;

    fn find_by_identity<'a>(
        &'a self,
        issuer: &'a Issuer,
        subject: &'a Subject,
    ) -> BoxFuture<'a, Result<Option<ProvisionedIdentity>, StoreError>>;
}
