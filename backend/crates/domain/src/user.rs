use crate::actor::ActorId;
use crate::status::status_conversion;
use crate::validation::{AvatarUrl, DisplayName, EmailAddress, Username, ValidationError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Disabled,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum UserError {
    #[error("invalid display name: {0}")]
    InvalidDisplayName(#[from] ValidationError),
    #[error("user is already disabled")]
    AlreadyDisabled,
    #[error("user is not disabled")]
    NotDisabled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: ActorId,
    pub display_name: DisplayName,
    pub email: EmailAddress,
    pub username: Username,
    pub status: UserStatus,
    pub avatar_url: Option<AvatarUrl>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl User {
    #[must_use]
    pub fn create(
        display_name: DisplayName,
        email: EmailAddress,
        username: Username,
        now: OffsetDateTime,
    ) -> Self {
        Self {
            id: ActorId::new(),
            display_name,
            email,
            username,
            status: UserStatus::Active,
            avatar_url: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn rename(&mut self, display_name: DisplayName, now: OffsetDateTime) {
        self.display_name = display_name;
        self.updated_at = now;
    }

    /// Set or clear the user's own avatar (an S3 link). Provider avatars live
    /// on the identity and are resolved separately.
    pub fn set_avatar_url(&mut self, avatar_url: Option<AvatarUrl>, now: OffsetDateTime) {
        self.avatar_url = avatar_url;
        self.updated_at = now;
    }

    pub fn disable(&mut self, now: OffsetDateTime) -> Result<(), UserError> {
        if self.status == UserStatus::Disabled {
            return Err(UserError::AlreadyDisabled);
        }
        self.status = UserStatus::Disabled;
        self.updated_at = now;
        Ok(())
    }

    pub fn enable(&mut self, now: OffsetDateTime) -> Result<(), UserError> {
        if self.status == UserStatus::Active {
            return Err(UserError::NotDisabled);
        }
        self.status = UserStatus::Active;
        self.updated_at = now;
        Ok(())
    }
}

status_conversion!(UserStatus, "user", { Active => "active", Disabled => "disabled" });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_storage_names_are_stable() {
        assert_eq!(UserStatus::Active.as_str(), "active");
        assert_eq!(UserStatus::Disabled.as_str(), "disabled");
        assert_eq!(
            UserStatus::try_from("disabled").expect("known status"),
            UserStatus::Disabled
        );
        assert!(UserStatus::try_from("deleted").is_err());
    }

    #[test]
    fn disable_and_enable_are_state_checked() {
        let now = OffsetDateTime::now_utc();
        let mut user = User::create(
            DisplayName::new("Ada".to_owned()).expect("valid"),
            EmailAddress::new("ada@example.test".to_owned()).expect("email"),
            Username::new("ada".to_owned()).expect("username"),
            now,
        );
        assert!(user.disable(now).is_ok());
        assert!(user.disable(now).is_err());
        assert!(user.enable(now).is_ok());
        assert!(user.enable(now).is_err());
    }
}
