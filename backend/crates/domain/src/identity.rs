use crate::ids::entity_id;
use crate::user::UserId;
use crate::validation::{AvatarUrl, DisplayName, EmailAddress, Issuer, Subject};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

entity_id!(IdentityId);

/// A login identity from an external provider.
///
/// Identity is keyed by `(issuer, subject)`. Email is provider-asserted
/// metadata and is never a linking key, so an unverified email cannot take
/// over another user's account.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    pub id: IdentityId,
    pub user_id: UserId,
    pub issuer: Issuer,
    pub subject: Subject,
    pub email: Option<EmailAddress>,
    pub email_verified: bool,
    pub display_name: Option<DisplayName>,
    pub avatar_url: Option<AvatarUrl>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Identity {
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the identity's stored columns"
    )]
    pub fn create(
        user_id: UserId,
        issuer: Issuer,
        subject: Subject,
        display_name: Option<DisplayName>,
        email: Option<EmailAddress>,
        email_verified: bool,
        avatar_url: Option<AvatarUrl>,
        now: OffsetDateTime,
    ) -> Self {
        Self {
            id: IdentityId::new(),
            user_id,
            issuer,
            subject,
            email,
            email_verified,
            display_name,
            avatar_url,
            created_at: now,
            updated_at: now,
        }
    }

    /// Refresh provider-asserted profile fields without changing the key.
    ///
    /// A provider that omits a field does not erase the stored value; only
    /// fields actually present in the claims are replaced.
    pub fn refresh_profile(
        &mut self,
        display_name: Option<DisplayName>,
        email: Option<EmailAddress>,
        email_verified: bool,
        avatar_url: Option<AvatarUrl>,
        now: OffsetDateTime,
    ) {
        if display_name.is_some() {
            self.display_name = display_name;
        }
        if email.is_some() {
            self.email = email;
            self.email_verified = email_verified;
        }
        if avatar_url.is_some() {
            self.avatar_url = avatar_url;
        }
        self.updated_at = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> Identity {
        let now = OffsetDateTime::now_utc();
        Identity::create(
            UserId::new(),
            Issuer::new("https://auth.example/realms/rastai".to_owned()).expect("issuer"),
            Subject::new("subject-1".to_owned()).expect("subject"),
            Some(DisplayName::new("Ada".to_owned()).expect("name")),
            Some(EmailAddress::new("ada@example.com".to_owned()).expect("email")),
            true,
            Some(AvatarUrl::new("https://example.com/a.png".to_owned()).expect("avatar")),
            now,
        )
    }

    #[test]
    fn refresh_replaces_only_present_fields() {
        let mut identity = identity();
        let created = identity.created_at;
        let later = created + time::Duration::seconds(60);

        identity.refresh_profile(None, None, false, None, later);

        assert_eq!(
            identity.display_name.as_ref().map(DisplayName::as_str),
            Some("Ada")
        );
        assert_eq!(
            identity.email.as_ref().map(EmailAddress::as_str),
            Some("ada@example.com")
        );
        assert!(identity.email_verified);
        assert!(identity.avatar_url.is_some());
        assert_eq!(identity.updated_at, later);
        assert_eq!(identity.created_at, created);
    }
}
