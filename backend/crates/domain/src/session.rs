use crate::ids::entity_id;
use crate::room::RoomId;
use crate::status::status_conversion;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

entity_id!(SessionId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Scheduled,
    Live,
    Ended,
    Cancelled,
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

/// A live occurrence of a [`Room`](crate::Room).
///
/// A session is the generic realtime lifecycle primitive (ADR 0020); the
/// storage table is `mahfel.sessions`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub room_id: RoomId,
    pub status: SessionStatus,
    pub scheduled_start_at: Option<OffsetDateTime>,
    pub started_at: Option<OffsetDateTime>,
    pub ended_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Session {
    #[must_use]
    pub fn schedule(
        room_id: RoomId,
        scheduled_start_at: Option<OffsetDateTime>,
        now: OffsetDateTime,
    ) -> Self {
        Self {
            id: SessionId::new(),
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
    fn status_storage_names_are_stable() {
        assert_eq!(SessionStatus::Scheduled.as_str(), "scheduled");
        assert_eq!(SessionStatus::Live.as_str(), "live");
        assert_eq!(SessionStatus::Ended.as_str(), "ended");
        assert_eq!(SessionStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(
            SessionStatus::try_from("live").expect("known status"),
            SessionStatus::Live
        );
        assert!(SessionStatus::try_from("unknown").is_err());
    }

    #[test]
    fn lifecycle_is_state_checked() {
        let now = OffsetDateTime::now_utc();
        let mut session = Session::schedule(RoomId::new(), None, now);
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
    fn cancel_only_from_scheduled() {
        let now = OffsetDateTime::now_utc();
        let mut session = Session::schedule(RoomId::new(), None, now);
        assert!(session.cancel(now).is_ok());
        assert_eq!(session.status, SessionStatus::Cancelled);
        let mut live = Session::schedule(RoomId::new(), None, now);
        live.start(now).expect("start");
        assert!(live.cancel(now).is_err());
    }
}
