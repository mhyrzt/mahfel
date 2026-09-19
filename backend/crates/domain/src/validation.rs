use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum lengths mirror the `CHECK (char_length(...))` bounds in
/// `migrations/0002_user_room_session.sql`. The database remains a
/// defensive constraint; these types are the authoritative validation.
pub const MAX_DISPLAY_NAME_LEN: usize = 200;
pub const MAX_ROOM_TITLE_LEN: usize = 300;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ValidationError {
    #[error("value must not be empty")]
    Empty,
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
}
