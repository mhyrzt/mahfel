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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub display_name: String,
    pub status: UserStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub title: String,
    pub created_by: UserId,
    pub status: RoomStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
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
}
