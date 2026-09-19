use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum lengths mirror the `CHECK (char_length(...))` bounds in
/// `migrations/0002_user_room_session.sql`. The database remains a
/// defensive constraint; these types are the authoritative validation.
pub const MAX_DISPLAY_NAME_LEN: usize = 200;
pub const MAX_ROOM_TITLE_LEN: usize = 300;
pub const MAX_ISSUER_LEN: usize = 500;
pub const MAX_SUBJECT_LEN: usize = 500;
pub const MAX_EMAIL_LEN: usize = 320;
pub const MAX_AVATAR_URL_LEN: usize = 2048;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ValidationError {
    #[error("value must not be empty")]
    Empty,
    #[error("value has an invalid format")]
    InvalidFormat,
    #[error("value exceeds {max} characters (got {got})")]
    TooLong { max: usize, got: usize },
}

/// A non-empty, trimmed, length-bounded text value.
fn validate_text(value: String, max: usize) -> Result<String, ValidationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::Empty);
    }
    let len = trimmed.chars().count();
    if len > max {
        return Err(ValidationError::TooLong { max, got: len });
    }
    Ok(trimmed.to_owned())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DisplayName(String);

impl DisplayName {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        validate_text(value, MAX_DISPLAY_NAME_LEN).map(Self)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoomTitle(String);

impl RoomTitle {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        validate_text(value, MAX_ROOM_TITLE_LEN).map(Self)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// OIDC issuer identifier. Stable per provider and part of the identity key.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Issuer(String);

impl Issuer {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        validate_text(value, MAX_ISSUER_LEN).map(Self)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// OIDC subject identifier, unique within an issuer and never an email.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Subject(String);

impl Subject {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        validate_text(value, MAX_SUBJECT_LEN).map(Self)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Provider-asserted email address. Verification is tracked separately.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        let trimmed = validate_text(value, MAX_EMAIL_LEN)?;
        let Some((local, domain)) = trimmed.split_once('@') else {
            return Err(ValidationError::InvalidFormat);
        };
        if local.is_empty() || domain.is_empty() || domain.contains('@') {
            return Err(ValidationError::InvalidFormat);
        }
        Ok(Self(trimmed))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Avatar location. Only HTTP(S) links are accepted, so unsafe schemes cannot
/// reach an image element.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AvatarUrl(String);

impl AvatarUrl {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        let trimmed = validate_text(value, MAX_AVATAR_URL_LEN)?;
        if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
            return Err(ValidationError::InvalidFormat);
        }
        Ok(Self(trimmed))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_validation() {
        assert!(DisplayName::new(String::new()).is_err());
        assert!(DisplayName::new("  ".to_owned()).is_err());
        assert!(DisplayName::new("ok".to_owned()).is_ok());
        let long = "a".repeat(MAX_DISPLAY_NAME_LEN + 1);
        assert!(DisplayName::new(long).is_err());
    }

    #[test]
    fn room_title_validation() {
        assert!(RoomTitle::new(String::new()).is_err());
        assert!(RoomTitle::new("ok".to_owned()).is_ok());
        let long = "a".repeat(MAX_ROOM_TITLE_LEN + 1);
        assert!(RoomTitle::new(long).is_err());
    }

    #[test]
    fn values_are_trimmed() {
        assert_eq!(
            DisplayName::new("  Ada  ".to_owned())
                .expect("valid")
                .as_str(),
            "Ada"
        );
    }

    #[test]
    fn identity_value_objects_validate() {
        assert!(Issuer::new("https://auth.mahfel.localhost/realms/rastai".to_owned()).is_ok());
        assert!(Issuer::new("   ".to_owned()).is_err());
        assert!(Subject::new("00000000-0000-4000-8000-000000000001".to_owned()).is_ok());
        assert!(Subject::new(String::new()).is_err());
    }

    #[test]
    fn email_address_requires_a_local_and_domain_part() {
        assert_eq!(
            EmailAddress::new("  ada@example.com ".to_owned())
                .expect("valid")
                .as_str(),
            "ada@example.com"
        );
        assert!(EmailAddress::new("not-an-email".to_owned()).is_err());
        assert!(EmailAddress::new("@example.com".to_owned()).is_err());
        assert!(EmailAddress::new("ada@".to_owned()).is_err());
        assert!(EmailAddress::new("a@b@c".to_owned()).is_err());
    }

    #[test]
    fn avatar_url_rejects_unsafe_schemes() {
        assert!(AvatarUrl::new("https://s3.mahfel.localhost/avatars/a.png".to_owned()).is_ok());
        assert!(AvatarUrl::new("http://example.com/a.png".to_owned()).is_ok());
        assert!(AvatarUrl::new("javascript:alert(1)".to_owned()).is_err());
        assert!(AvatarUrl::new("data:image/png;base64,AAAA".to_owned()).is_err());
        assert!(AvatarUrl::new("/relative/avatar.png".to_owned()).is_err());
    }
}
