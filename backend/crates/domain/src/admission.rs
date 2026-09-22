//! Admission control: the generic form of a waiting room (ADR 0020).
//!
//! A room carries an admission policy. Under [`AdmissionPolicy::Open`] an
//! authorized principal joins immediately; under [`AdmissionPolicy::Lobby`] the
//! same principal becomes a pending request that a principal holding
//! `admit_participant` must decide.
//!
//! Admission never replaces authorization. It layers on top: a caller who fails
//! the join guard is still denied, and a caller awaiting a decision holds no
//! participation capability and has no participant identity to publish as.

use crate::actor::ActorId;
use crate::ids::entity_id;
use crate::session::SessionId;
use crate::status::status_conversion;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

entity_id!(AdmissionRequestId);

/// Whether joining a target requires approval.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionPolicy {
    /// An authorized caller joins immediately. The default, and the behavior
    /// every existing room already has.
    #[default]
    Open,
    /// An authorized caller waits for a decision.
    Lobby,
}

status_conversion!(AdmissionPolicy, "admission policy", {
    Open => "open",
    Lobby => "lobby",
});

impl AdmissionPolicy {
    /// Whether joining requires a decision. Named so call sites read as intent
    /// rather than as an enum comparison.
    #[must_use]
    pub const fn requires_approval(self) -> bool {
        matches!(self, Self::Lobby)
    }

    /// The stricter of two policies.
    ///
    /// An override may only tighten access, never loosen it, so a session can
    /// never expose a room that its room would have protected.
    #[must_use]
    pub const fn stricter(self, other: Self) -> Self {
        match (self, other) {
            (Self::Lobby, _) | (_, Self::Lobby) => Self::Lobby,
            _ => Self::Open,
        }
    }
}

/// Lifecycle of one principal's request to join a session.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionRequestStatus {
    Pending,
    Admitted,
    Rejected,
    /// The session ended before anyone decided. Distinct from `Rejected`
    /// because no principal made the call, and the audit trail must not imply
    /// one did.
    Expired,
}

status_conversion!(AdmissionRequestStatus, "admission request status", {
    Pending => "pending",
    Admitted => "admitted",
    Rejected => "rejected",
    Expired => "expired",
});

/// Where a request came from, so an operator can tell an explicit join from an
/// invite-link redemption (ADR 0021).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionOrigin {
    Join,
    InviteLink,
}

status_conversion!(AdmissionOrigin, "admission origin", {
    Join => "join",
    InviteLink => "invite_link",
});

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AdmissionError {
    #[error("admission request was already decided")]
    AlreadyDecided,
    #[error("lobby is at capacity")]
    LobbyFull,
}

/// One principal's outstanding request to join one session.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdmissionRequest {
    pub id: AdmissionRequestId,
    pub session_id: SessionId,
    pub actor_id: ActorId,
    pub status: AdmissionRequestStatus,
    pub origin: AdmissionOrigin,
    pub requested_at: OffsetDateTime,
    pub decided_at: Option<OffsetDateTime>,
    pub decided_by: Option<ActorId>,
}

impl AdmissionRequest {
    #[must_use]
    pub fn new(
        session_id: SessionId,
        actor_id: ActorId,
        origin: AdmissionOrigin,
        now: OffsetDateTime,
    ) -> Self {
        Self {
            id: AdmissionRequestId::new(),
            session_id,
            actor_id,
            status: AdmissionRequestStatus::Pending,
            origin,
            requested_at: now,
            decided_at: None,
            decided_by: None,
        }
    }

    /// Approve the request. Only a pending request may be decided.
    ///
    /// # Errors
    /// [`AdmissionError::AlreadyDecided`] when the request is not pending, so a
    /// duplicate decision cannot overwrite the first one's principal or time.
    pub fn admit(&mut self, by: ActorId, now: OffsetDateTime) -> Result<(), AdmissionError> {
        self.decide(AdmissionRequestStatus::Admitted, Some(by), now)
    }

    /// Reject the request. Terminates the waiting state.
    ///
    /// # Errors
    /// [`AdmissionError::AlreadyDecided`] when the request is not pending.
    pub fn reject(&mut self, by: ActorId, now: OffsetDateTime) -> Result<(), AdmissionError> {
        self.decide(AdmissionRequestStatus::Rejected, Some(by), now)
    }

    /// Retire a request because its session ended. No principal decided it.
    ///
    /// # Errors
    /// [`AdmissionError::AlreadyDecided`] when the request is not pending.
    pub fn expire(&mut self, now: OffsetDateTime) -> Result<(), AdmissionError> {
        self.decide(AdmissionRequestStatus::Expired, None, now)
    }

    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.status == AdmissionRequestStatus::Pending
    }

    fn decide(
        &mut self,
        status: AdmissionRequestStatus,
        by: Option<ActorId>,
        now: OffsetDateTime,
    ) -> Result<(), AdmissionError> {
        if !self.is_pending() {
            return Err(AdmissionError::AlreadyDecided);
        }
        self.status = status;
        self.decided_at = Some(now);
        self.decided_by = by;
        Ok(())
    }
}

/// Most principals that may wait in one session's lobby at once.
///
/// An unbounded waiting set is a flooding vector: any caller who passes the
/// join guard could otherwise fill a manager's queue.
pub const MAX_PENDING_PER_SESSION: usize = 50;

#[cfg(test)]
mod tests {
    use super::*;

    fn at(seconds: i64) -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(seconds)
    }

    fn request() -> AdmissionRequest {
        AdmissionRequest::new(
            SessionId::new(),
            ActorId::new(),
            AdmissionOrigin::Join,
            at(0),
        )
    }

    #[test]
    fn policy_storage_names_are_stable() {
        assert_eq!(AdmissionPolicy::Open.as_str(), "open");
        assert_eq!(AdmissionPolicy::Lobby.as_str(), "lobby");
        assert!(AdmissionPolicy::try_from("waiting").is_err());
        assert_eq!(AdmissionPolicy::default(), AdmissionPolicy::Open);
    }

    #[test]
    fn request_status_and_origin_names_are_stable() {
        assert_eq!(AdmissionRequestStatus::Pending.as_str(), "pending");
        assert_eq!(AdmissionRequestStatus::Admitted.as_str(), "admitted");
        assert_eq!(AdmissionRequestStatus::Rejected.as_str(), "rejected");
        assert_eq!(AdmissionRequestStatus::Expired.as_str(), "expired");
        assert_eq!(AdmissionOrigin::Join.as_str(), "join");
        assert_eq!(AdmissionOrigin::InviteLink.as_str(), "invite_link");
    }

    #[test]
    fn only_lobby_requires_approval() {
        assert!(!AdmissionPolicy::Open.requires_approval());
        assert!(AdmissionPolicy::Lobby.requires_approval());
    }

    #[test]
    fn stricter_never_yields_the_looser_policy() {
        for left in [AdmissionPolicy::Open, AdmissionPolicy::Lobby] {
            for right in [AdmissionPolicy::Open, AdmissionPolicy::Lobby] {
                let resolved = left.stricter(right);
                if left == AdmissionPolicy::Lobby || right == AdmissionPolicy::Lobby {
                    assert_eq!(resolved, AdmissionPolicy::Lobby);
                } else {
                    assert_eq!(resolved, AdmissionPolicy::Open);
                }
            }
        }
    }

    #[test]
    fn a_new_request_is_pending_and_undecided() {
        let request = request();
        assert!(request.is_pending());
        assert!(request.decided_at.is_none());
        assert!(request.decided_by.is_none());
    }

    #[test]
    fn admit_records_the_deciding_principal() {
        let mut request = request();
        let manager = ActorId::new();
        request.admit(manager, at(30)).expect("admit");
        assert_eq!(request.status, AdmissionRequestStatus::Admitted);
        assert_eq!(request.decided_by, Some(manager));
        assert_eq!(request.decided_at, Some(at(30)));
    }

    #[test]
    fn reject_terminates_the_waiting_state() {
        let mut request = request();
        request.reject(ActorId::new(), at(30)).expect("reject");
        assert_eq!(request.status, AdmissionRequestStatus::Rejected);
        assert!(!request.is_pending());
    }

    #[test]
    fn expiry_names_no_deciding_principal() {
        let mut request = request();
        request.expire(at(90)).expect("expire");
        assert_eq!(request.status, AdmissionRequestStatus::Expired);
        assert!(
            request.decided_by.is_none(),
            "no principal decided an expiry"
        );
        assert_eq!(request.decided_at, Some(at(90)));
    }

    #[test]
    fn a_decided_request_cannot_be_decided_again() {
        for decide in [
            AdmissionRequest::admit as fn(&mut AdmissionRequest, ActorId, OffsetDateTime) -> _,
            AdmissionRequest::reject,
        ] {
            let mut request = request();
            decide(&mut request, ActorId::new(), at(10)).expect("first decision");
            let recorded = (request.status, request.decided_at, request.decided_by);
            assert_eq!(
                decide(&mut request, ActorId::new(), at(20)),
                Err(AdmissionError::AlreadyDecided)
            );
            assert_eq!(
                (request.status, request.decided_at, request.decided_by),
                recorded,
                "a repeat must not overwrite the recorded decision"
            );
        }
    }

    #[test]
    fn pending_requests_have_no_decision_record() {
        let request = request();
        assert_eq!(
            request.is_pending(),
            request.decided_at.is_none() && request.decided_by.is_none()
        );
    }
}
