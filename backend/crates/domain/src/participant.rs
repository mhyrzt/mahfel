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
    #[error("participant is not muted")]
    NotMuted,
    #[error("a permanent mute can only be cleared by a moderator")]
    PermanentMuteNeedsModerator,
}

/// How firmly a participant is muted.
///
/// The two are not degrees of the same thing: a temporary mute is a reset the
/// target clears themselves, while a permanent mute is held until someone with
/// the capability lifts it. Neither survives a rejoin, because leaving is
/// terminal; keeping someone out after they leave is a block, not a mute.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MuteKind {
    /// The target may clear it.
    Temporary,
    /// Only a moderator may clear it.
    Permanent,
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
    /// When the current mute was applied. `mute_kind` is `None` if and only if
    /// this is `None`, so there is no reachable muted-without-a-kind state.
    pub muted_at: Option<OffsetDateTime>,
    pub mute_kind: Option<MuteKind>,
    /// Who applied the current mute.
    pub muted_by: Option<ActorId>,
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
            muted_at: None,
            mute_kind: None,
            muted_by: None,
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

    /// Mute this participant's audio.
    ///
    /// Re-muting replaces the previous kind, which is what makes an upgrade
    /// from temporary to permanent meaningful rather than a conflict. Only an
    /// active participant can be muted: a mute on someone who has left would
    /// apply to nobody.
    ///
    /// # Errors
    /// [`ParticipantError::AlreadyLeft`] or [`ParticipantError::Removed`] when
    /// the participant is no longer active.
    pub fn mute(
        &mut self,
        kind: MuteKind,
        by: ActorId,
        now: OffsetDateTime,
    ) -> Result<(), ParticipantError> {
        self.require_active()?;
        self.muted_at = Some(now);
        self.mute_kind = Some(kind);
        self.muted_by = Some(by);
        self.updated_at = now;
        Ok(())
    }

    /// Clear a mute.
    ///
    /// A temporary mute is the target's own to clear. A permanent one is not:
    /// `may_moderate` must be true, which the caller determines by resolving
    /// the capability. The rule lives here because it is a state-transition
    /// invariant, not a transport concern.
    ///
    /// # Errors
    /// [`ParticipantError::NotMuted`] when there is nothing to clear,
    /// [`ParticipantError::PermanentMuteNeedsModerator`] on a self-unmute of a
    /// permanent mute.
    pub fn unmute(
        &mut self,
        may_moderate: bool,
        now: OffsetDateTime,
    ) -> Result<(), ParticipantError> {
        match self.mute_kind {
            None => Err(ParticipantError::NotMuted),
            Some(MuteKind::Temporary) => {
                self.clear_mute(now);
                Ok(())
            }
            Some(MuteKind::Permanent) if may_moderate => {
                self.clear_mute(now);
                Ok(())
            }
            Some(MuteKind::Permanent) => Err(ParticipantError::PermanentMuteNeedsModerator),
        }
    }

    fn clear_mute(&mut self, now: OffsetDateTime) {
        self.muted_at = None;
        self.mute_kind = None;
        self.muted_by = None;
        self.updated_at = now;
    }

    fn require_active(&self) -> Result<(), ParticipantError> {
        match self.status {
            ParticipantStatus::Active => Ok(()),
            ParticipantStatus::Left => Err(ParticipantError::AlreadyLeft),
            ParticipantStatus::Removed => Err(ParticipantError::Removed),
        }
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status == ParticipantStatus::Active
    }

    #[must_use]
    pub fn is_muted(&self) -> bool {
        self.mute_kind.is_some()
    }
}

status_conversion!(ParticipantStatus, "participant", {
    Active => "active",
    Left => "left",
    Removed => "removed",
});

status_conversion!(MuteKind, "mute kind", {
    Temporary => "temporary",
    Permanent => "permanent",
});

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> Participant {
        Participant::join(SessionId::new(), ActorId::new(), OffsetDateTime::now_utc())
    }

    /// A fixed instant, so a test can assert which time a transition recorded.
    fn at(seconds: i64) -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(seconds)
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
    fn mute_kind_storage_names_are_stable() {
        assert_eq!(MuteKind::Temporary.as_str(), "temporary");
        assert_eq!(MuteKind::Permanent.as_str(), "permanent");
        assert_eq!(
            MuteKind::try_from("permanent").expect("known kind"),
            MuteKind::Permanent
        );
        assert!(MuteKind::try_from("forever").is_err());
    }

    #[test]
    fn a_new_participant_is_not_muted() {
        let participant = active();
        assert!(!participant.is_muted());
        assert!(participant.mute_kind.is_none());
        assert!(participant.muted_at.is_none());
        assert!(participant.muted_by.is_none());
    }

    #[test]
    fn a_mute_records_its_kind_and_principal() {
        let mut participant = active();
        let moderator = ActorId::new();
        participant
            .mute(MuteKind::Temporary, moderator, at(10))
            .expect("mute");

        assert!(participant.is_muted());
        assert_eq!(participant.mute_kind, Some(MuteKind::Temporary));
        assert_eq!(participant.muted_by, Some(moderator));
        assert_eq!(participant.muted_at, Some(at(10)));
    }

    #[test]
    fn the_target_clears_their_own_temporary_mute() {
        let mut participant = active();
        participant
            .mute(MuteKind::Temporary, ActorId::new(), at(10))
            .expect("mute");
        participant.unmute(false, at(20)).expect("unmute");

        assert!(!participant.is_muted());
        assert!(participant.muted_at.is_none());
        assert!(participant.muted_by.is_none());
    }

    #[test]
    fn the_target_cannot_clear_a_permanent_mute() {
        let mut participant = active();
        participant
            .mute(MuteKind::Permanent, ActorId::new(), at(10))
            .expect("mute");

        assert_eq!(
            participant.unmute(false, at(20)),
            Err(ParticipantError::PermanentMuteNeedsModerator),
            "a permanent mute is not the target's to clear"
        );
        assert!(
            participant.is_muted(),
            "a refused unmute must not change the state"
        );
    }

    #[test]
    fn a_moderator_clears_a_permanent_mute() {
        let mut participant = active();
        participant
            .mute(MuteKind::Permanent, ActorId::new(), at(10))
            .expect("mute");
        participant.unmute(true, at(20)).expect("moderator unmute");

        assert!(!participant.is_muted());
    }

    #[test]
    fn unmuting_something_that_is_not_muted_is_refused() {
        let mut participant = active();
        assert_eq!(
            participant.unmute(true, at(20)),
            Err(ParticipantError::NotMuted)
        );
    }

    #[test]
    fn re_muting_replaces_the_kind_rather_than_conflicting() {
        let mut participant = active();
        participant
            .mute(MuteKind::Temporary, ActorId::new(), at(10))
            .expect("mute");
        participant
            .mute(MuteKind::Permanent, ActorId::new(), at(20))
            .expect("upgrade to permanent");

        assert_eq!(
            participant.mute_kind,
            Some(MuteKind::Permanent),
            "an upgrade to permanent must take effect, not be ignored"
        );
    }

    #[test]
    fn a_muted_participant_can_still_leave() {
        // Mute is about media, not standing: it must not block the lifecycle.
        let mut participant = active();
        participant
            .mute(MuteKind::Permanent, ActorId::new(), at(10))
            .expect("mute");
        participant.leave(at(20)).expect("leave");
        assert!(!participant.is_active());
    }

    #[test]
    fn a_participant_who_left_cannot_be_muted() {
        let mut participant = active();
        participant.leave(at(10)).expect("leave");
        assert_eq!(
            participant.mute(MuteKind::Temporary, ActorId::new(), at(20)),
            Err(ParticipantError::AlreadyLeft),
            "muting someone who has left would apply to nobody"
        );
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
