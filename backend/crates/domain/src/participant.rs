use crate::actor::ActorId;
use crate::ids::entity_id;
use crate::session::SessionId;
use crate::status::status_conversion;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

entity_id!(ParticipantId);

/// Lifecycle of one actor's presence in one session.
///
/// `Left` and `Removed` are terminal: a later join creates a new participant
/// rather than reviving the row, so participation history stays append-only.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantStatus {
    Active,
    Left,
    Removed,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ParticipantError {
    #[error("participant has left")]
    AlreadyLeft,
    #[error("participant was removed")]
    Removed,
    #[error("participant is already active")]
    AlreadyActive,
}

/// A durable identity for one actor (user or service principal) inside one
/// session, keyed by `(session_id, actor_id)` rather than by connection.
///
/// A reconnect or a second device reuses this row instead of creating another,
/// so the roster and moderation target a stable identity across transport
/// churn. Connection state — how many transports are attached, and which is
/// newest — is ephemeral and lives in the realtime registry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Participant {
    pub id: ParticipantId,
    pub session_id: SessionId,
    pub actor_id: ActorId,
    pub status: ParticipantStatus,
    pub joined_at: OffsetDateTime,
    pub left_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Participant {
    /// Callers must already have established that the actor has no active
    /// participant for this session; the partial unique index makes a racing
    /// join fail as a conflict instead of duplicating the row.
    #[must_use]
    pub fn join(session_id: SessionId, actor_id: ActorId, now: OffsetDateTime) -> Self {
        Self {
            id: ParticipantId::new(),
            session_id,
            actor_id,
            status: ParticipantStatus::Active,
            joined_at: now,
            left_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn leave(&mut self, now: OffsetDateTime) -> Result<(), ParticipantError> {
        match self.status {
            ParticipantStatus::Active => {
                self.status = ParticipantStatus::Left;
                self.left_at = Some(now);
                self.updated_at = now;
                Ok(())
            }
            ParticipantStatus::Left => Err(ParticipantError::AlreadyLeft),
            ParticipantStatus::Removed => Err(ParticipantError::Removed),
        }
    }

    pub fn remove(&mut self, now: OffsetDateTime) -> Result<(), ParticipantError> {
        match self.status {
            ParticipantStatus::Active => {
                self.status = ParticipantStatus::Removed;
                self.left_at = Some(now);
                self.updated_at = now;
                Ok(())
            }
            ParticipantStatus::Left => Err(ParticipantError::AlreadyLeft),
            ParticipantStatus::Removed => Err(ParticipantError::Removed),
        }
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status == ParticipantStatus::Active
    }
}

status_conversion!(ParticipantStatus, "participant", {
    Active => "active",
    Left => "left",
    Removed => "removed",
});

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> Participant {
        Participant::join(SessionId::new(), ActorId::new(), OffsetDateTime::now_utc())
    }

    #[test]
    fn status_storage_names_are_stable() {
        assert_eq!(ParticipantStatus::Active.as_str(), "active");
        assert_eq!(ParticipantStatus::Left.as_str(), "left");
        assert_eq!(ParticipantStatus::Removed.as_str(), "removed");
        assert_eq!(
            ParticipantStatus::try_from("removed").expect("known status"),
            ParticipantStatus::Removed
        );
        assert!(ParticipantStatus::try_from("kicked").is_err());
    }

    #[test]
    fn join_starts_active_without_a_leave_time() {
        let participant = active();
        assert!(participant.is_active());
        assert!(participant.left_at.is_none());
        assert_eq!(participant.joined_at, participant.created_at);
    }

    #[test]
    fn leave_is_terminal() {
        let now = OffsetDateTime::now_utc();
        let mut participant = active();
        assert!(participant.leave(now).is_ok());
        assert_eq!(participant.status, ParticipantStatus::Left);
        assert_eq!(participant.left_at, Some(now));
        assert_eq!(
            participant.leave(now).expect_err("already left"),
            ParticipantError::AlreadyLeft
        );
        assert_eq!(
            participant.remove(now).expect_err("already left"),
            ParticipantError::AlreadyLeft
        );
    }

    #[test]
    fn removal_is_terminal_and_distinct_from_leaving() {
        let now = OffsetDateTime::now_utc();
        let mut participant = active();
        assert!(participant.remove(now).is_ok());
        assert_eq!(participant.status, ParticipantStatus::Removed);
        assert_eq!(
            participant.remove(now).expect_err("already removed"),
            ParticipantError::Removed
        );
        assert_eq!(
            participant.leave(now).expect_err("already removed"),
            ParticipantError::Removed
        );
    }

    #[test]
    fn the_same_actor_may_hold_two_participants_in_different_sessions() {
        let actor = ActorId::new();
        let now = OffsetDateTime::now_utc();
        let first = Participant::join(SessionId::new(), actor, now);
        let second = Participant::join(SessionId::new(), actor, now);
        assert_ne!(first.id, second.id);
        assert_ne!(first.session_id, second.session_id);
    }
}
