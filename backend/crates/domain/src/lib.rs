use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

macro_rules! entity_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            #[must_use]
            pub const fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

entity_id!(UserId);
entity_id!(RoomId);
entity_id!(ClassroomSessionId);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(String);

impl NodeId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomStatus {
    Active,
    Archived,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Scheduled,
    Live,
    Ended,
    Cancelled,
}

/// Maximum lengths mirror the `CHECK (char_length(...))` bounds in
/// `migrations/0002_user_room_session.sql`. The database remains a
/// defensive constraint; these types are the authoritative validation.
pub const MAX_DISPLAY_NAME_LEN: usize = 200;
pub const MAX_ROOM_TITLE_LEN: usize = 300;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DisplayName(String);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoomTitle(String);

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ValidationError {
    #[error("value must not be empty")]
    Empty,
    #[error("value exceeds {max} characters (got {got})")]
    TooLong { max: usize, got: usize },
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RoomError {
    #[error("room is already archived")]
    AlreadyArchived,
    #[error("invalid room title: {0}")]
    InvalidTitle(#[from] ValidationError),
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

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SessionError {
    #[error("session cannot start from status `{0}`")]
    CannotStart(&'static str),
    #[error("session cannot end from status `{0}`")]
    CannotEnd(&'static str),
    #[error("session cannot be cancelled from status `{0}`")]
    CannotCancel(&'static str),
}

impl DisplayName {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty);
        }
        let len = trimmed.chars().count();
        if len > MAX_DISPLAY_NAME_LEN {
            return Err(ValidationError::TooLong {
                max: MAX_DISPLAY_NAME_LEN,
                got: len,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl RoomTitle {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::Empty);
        }
        let len = trimmed.chars().count();
        if len > MAX_ROOM_TITLE_LEN {
            return Err(ValidationError::TooLong {
                max: MAX_ROOM_TITLE_LEN,
                got: len,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub display_name: DisplayName,
    pub status: UserStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl User {
    #[must_use]
    pub fn create(display_name: DisplayName, now: OffsetDateTime) -> Self {
        Self {
            id: UserId::new(),
            display_name,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn rename(&mut self, display_name: DisplayName, now: OffsetDateTime) {
        self.display_name = display_name;
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub title: RoomTitle,
    pub created_by: UserId,
    pub status: RoomStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Room {
    #[must_use]
    pub fn create(title: RoomTitle, created_by: UserId, now: OffsetDateTime) -> Self {
        Self {
            id: RoomId::new(),
            title,
            created_by,
            status: RoomStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn rename(&mut self, title: RoomTitle, now: OffsetDateTime) -> Result<(), RoomError> {
        if self.status == RoomStatus::Archived {
            return Err(RoomError::AlreadyArchived);
        }
        self.title = title;
        self.updated_at = now;
        Ok(())
    }

    pub fn archive(&mut self, now: OffsetDateTime) -> Result<(), RoomError> {
        if self.status == RoomStatus::Archived {
            return Err(RoomError::AlreadyArchived);
        }
        self.status = RoomStatus::Archived;
        self.updated_at = now;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassroomSession {
    pub id: ClassroomSessionId,
    pub room_id: RoomId,
    pub status: SessionStatus,
    pub scheduled_start_at: Option<OffsetDateTime>,
    pub started_at: Option<OffsetDateTime>,
    pub ended_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl ClassroomSession {
    #[must_use]
    pub fn schedule(
        room_id: RoomId,
        scheduled_start_at: Option<OffsetDateTime>,
        now: OffsetDateTime,
    ) -> Self {
        Self {
            id: ClassroomSessionId::new(),
            room_id,
            status: SessionStatus::Scheduled,
            scheduled_start_at,
            started_at: None,
            ended_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn start(&mut self, now: OffsetDateTime) -> Result<(), SessionError> {
        if self.status != SessionStatus::Scheduled {
            return Err(SessionError::CannotStart(self.status.as_str()));
        }
        self.status = SessionStatus::Live;
        self.started_at = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn end(&mut self, now: OffsetDateTime) -> Result<(), SessionError> {
        if self.status != SessionStatus::Live {
            return Err(SessionError::CannotEnd(self.status.as_str()));
        }
        self.status = SessionStatus::Ended;
        self.ended_at = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn cancel(&mut self, now: OffsetDateTime) -> Result<(), SessionError> {
        if self.status != SessionStatus::Scheduled {
            return Err(SessionError::CannotCancel(self.status.as_str()));
        }
        self.status = SessionStatus::Cancelled;
        self.updated_at = now;
        Ok(())
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("unsupported {kind} status `{value}`")]
pub struct InvalidStatus {
    kind: &'static str,
    value: String,
}

macro_rules! status_conversion {
    ($type:ty, $kind:literal, {$($variant:ident => $value:literal),+ $(,)?}) => {
        impl $type {
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }

        impl TryFrom<&str> for $type {
            type Error = InvalidStatus;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok(Self::$variant)),+,
                    _ => Err(InvalidStatus {
                        kind: $kind,
                        value: value.to_owned(),
                    }),
                }
            }
        }
    };
}

status_conversion!(UserStatus, "user", { Active => "active", Disabled => "disabled" });
status_conversion!(RoomStatus, "room", { Active => "active", Archived => "archived" });
status_conversion!(SessionStatus, "session", {
    Scheduled => "scheduled",
    Live => "live",
    Ended => "ended",
    Cancelled => "cancelled",
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_values_have_stable_storage_names() {
        assert_eq!(UserStatus::Active.as_str(), "active");
        assert_eq!(RoomStatus::Archived.as_str(), "archived");
        assert_eq!(SessionStatus::Scheduled.as_str(), "scheduled");
    }

    #[test]
    fn unknown_status_is_rejected() {
        assert!(SessionStatus::try_from("unknown").is_err());
    }

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
    fn room_archive_transitions() {
        let now = OffsetDateTime::now_utc();
        let mut room = Room::create(
            RoomTitle::new("title".to_owned()).expect("valid"),
            UserId::new(),
            now,
        );
        assert_eq!(room.status, RoomStatus::Active);
        assert!(room.archive(now).is_ok());
        assert_eq!(room.status, RoomStatus::Archived);
        assert!(room.archive(now).is_err());
    }

    #[test]
    fn session_lifecycle() {
        let now = OffsetDateTime::now_utc();
        let mut session = ClassroomSession::schedule(RoomId::new(), None, now);
        assert_eq!(session.status, SessionStatus::Scheduled);
        assert!(session.end(now).is_err());
        assert!(session.start(now).is_ok());
        assert_eq!(session.status, SessionStatus::Live);
        assert!(session.start(now).is_err());
        assert!(session.cancel(now).is_err());
        assert!(session.end(now).is_ok());
        assert_eq!(session.status, SessionStatus::Ended);
    }

    #[test]
    fn session_cancel_only_from_scheduled() {
        let now = OffsetDateTime::now_utc();
        let mut session = ClassroomSession::schedule(RoomId::new(), None, now);
        assert!(session.cancel(now).is_ok());
        assert_eq!(session.status, SessionStatus::Cancelled);
        let mut live = ClassroomSession::schedule(RoomId::new(), None, now);
        live.start(now).expect("start");
        assert!(live.cancel(now).is_err());
    }
}
