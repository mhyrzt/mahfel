use crate::admission::AdmissionPolicy;
use crate::ids::entity_id;
use crate::room::{Room, RoomId, RoomVisibility};
use crate::status::status_conversion;
use crate::validation::{SessionDurationMinutes, SessionTitle};
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

/// Who may reach a session, when it differs from its room.
///
/// A distinct type from [`RoomVisibility`] so the two axes cannot be assigned
/// across by accident.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionVisibility {
    Public,
    InviteOnly,
}

status_conversion!(SessionVisibility, "session visibility", {
    Public => "public",
    InviteOnly => "invite_only",
});

impl SessionVisibility {
    /// Whether an outsider may reach the session.
    ///
    /// Used by the "an override may only tighten" rule: it is the value an
    /// override must not loosen past its room's.
    #[must_use]
    pub const fn is_public(self) -> bool {
        matches!(self, Self::Public)
    }

    /// The stricter of two visibilities.
    #[must_use]
    pub const fn stricter(self, other: Self) -> Self {
        match (self, other) {
            (Self::InviteOnly, _) | (_, Self::InviteOnly) => Self::InviteOnly,
            _ => Self::Public,
        }
    }
}

impl From<RoomVisibility> for SessionVisibility {
    fn from(value: RoomVisibility) -> Self {
        match value {
            RoomVisibility::Public => Self::Public,
            RoomVisibility::InviteOnly => Self::InviteOnly,
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SessionError {
    #[error("session cannot start from status `{0}`")]
    CannotStart(&'static str),
    #[error("session cannot end from status `{0}`")]
    CannotEnd(&'static str),
    #[error("session cannot be cancelled from status `{0}`")]
    CannotCancel(&'static str),
    /// A session override may only tighten access, never loosen it.
    #[error("a session override cannot loosen the room's access settings")]
    CannotLoosen,
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
    /// Session-level override of the room's visibility. `None` inherits.
    pub visibility: Option<SessionVisibility>,
    /// Session-level override of the room's admission policy. `None` inherits.
    pub admission: Option<AdmissionPolicy>,
    pub title: Option<SessionTitle>,
    pub scheduled_start_at: Option<OffsetDateTime>,
    pub scheduled_duration_minutes: Option<SessionDurationMinutes>,
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
            visibility: None,
            admission: None,
            title: None,
            scheduled_start_at,
            scheduled_duration_minutes: None,
            started_at: None,
            ended_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Change the optional display details in any lifecycle state. A missing
    /// title inherits the room's current title at read time.
    pub fn set_details(
        &mut self,
        title: Option<SessionTitle>,
        scheduled_start_at: Option<OffsetDateTime>,
        scheduled_duration_minutes: Option<SessionDurationMinutes>,
        now: OffsetDateTime,
    ) {
        self.title = title;
        self.scheduled_start_at = scheduled_start_at;
        self.scheduled_duration_minutes = scheduled_duration_minutes;
        self.updated_at = now;
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

    /// Set or clear the visibility override.
    ///
    /// An override may only tighten access, so a session can never expose a
    /// room the room itself would have protected.
    ///
    /// # Errors
    /// [`SessionError::CannotLoosen`] when `visibility` would relax the room's.
    pub fn set_visibility(
        &mut self,
        visibility: Option<SessionVisibility>,
        room: &Room,
        now: OffsetDateTime,
    ) -> Result<(), SessionError> {
        if let Some(override_value) = visibility {
            let inherited = SessionVisibility::from(room.visibility);
            if !inherited.stricter(override_value).eq(&override_value) {
                return Err(SessionError::CannotLoosen);
            }
        }
        self.visibility = visibility;
        self.updated_at = now;
        Ok(())
    }

    /// Set or clear the admission override.
    ///
    /// # Errors
    /// [`SessionError::CannotLoosen`] when `admission` would relax the room's.
    pub fn set_admission(
        &mut self,
        admission: Option<AdmissionPolicy>,
        room: &Room,
        now: OffsetDateTime,
    ) -> Result<(), SessionError> {
        if let Some(override_value) = admission
            && !room.admission.stricter(override_value).eq(&override_value)
        {
            return Err(SessionError::CannotLoosen);
        }
        self.admission = admission;
        self.updated_at = now;
        Ok(())
    }

    /// The visibility that governs this session, resolving the override.
    #[must_use]
    pub fn effective_visibility(&self, room: &Room) -> SessionVisibility {
        self.visibility
            .unwrap_or_else(|| SessionVisibility::from(room.visibility))
    }

    /// The admission policy that governs this session, resolving the override.
    #[must_use]
    pub fn effective_admission(&self, room: &Room) -> AdmissionPolicy {
        self.admission.unwrap_or(room.admission)
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
    fn advisory_details_do_not_drive_lifecycle() {
        let now = OffsetDateTime::now_utc();
        let future = now + time::Duration::days(2);
        let mut session = Session::schedule(RoomId::new(), Some(future), now);
        session.set_details(
            Some(SessionTitle::new("  Seminar  ".to_owned()).expect("valid title")),
            Some(future),
            Some(SessionDurationMinutes::new(120).expect("valid duration")),
            now,
        );
        assert_eq!(
            session.title.as_ref().map(SessionTitle::as_str),
            Some("Seminar")
        );
        assert!(
            session.start(now).is_ok(),
            "planned start never gates start"
        );
        assert_eq!(session.status, SessionStatus::Live);
        session.end(now).expect("explicit end");
        session.set_details(None, None, None, now);
        assert_eq!(session.status, SessionStatus::Ended);
        assert!(session.title.is_none());
        assert!(session.scheduled_duration_minutes.is_none());
    }

    fn room_with(visibility: RoomVisibility, admission: AdmissionPolicy) -> Room {
        let mut room = Room::create(
            crate::validation::RoomTitle::new("Room".to_owned()).expect("title"),
            crate::ActorId::new(),
            visibility,
            OffsetDateTime::now_utc(),
        );
        room.set_admission(admission, OffsetDateTime::now_utc())
            .expect("admission");
        room
    }

    #[test]
    fn session_visibility_storage_names_are_stable() {
        assert_eq!(SessionVisibility::Public.as_str(), "public");
        assert_eq!(SessionVisibility::InviteOnly.as_str(), "invite_only");
        assert!(SessionVisibility::try_from("open").is_err());
    }

    #[test]
    fn an_unset_override_inherits_both_axes() {
        let room = room_with(RoomVisibility::Public, AdmissionPolicy::Lobby);
        let session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
        assert_eq!(
            session.effective_visibility(&room),
            SessionVisibility::Public
        );
        assert_eq!(session.effective_admission(&room), AdmissionPolicy::Lobby);
    }

    #[test]
    fn a_set_override_wins_over_the_room() {
        let now = OffsetDateTime::now_utc();
        let room = room_with(RoomVisibility::Public, AdmissionPolicy::Open);
        let mut session = Session::schedule(room.id, None, now);
        session
            .set_visibility(Some(SessionVisibility::InviteOnly), &room, now)
            .expect("tighten");
        session
            .set_admission(Some(AdmissionPolicy::Lobby), &room, now)
            .expect("tighten");
        assert_eq!(
            session.effective_visibility(&room),
            SessionVisibility::InviteOnly
        );
        assert_eq!(session.effective_admission(&room), AdmissionPolicy::Lobby);
    }

    #[test]
    fn an_override_may_only_tighten() {
        let now = OffsetDateTime::now_utc();
        let room = room_with(RoomVisibility::InviteOnly, AdmissionPolicy::Lobby);
        let mut session = Session::schedule(room.id, None, now);
        assert_eq!(
            session.set_visibility(Some(SessionVisibility::Public), &room, now),
            Err(SessionError::CannotLoosen),
            "a session must not expose a room the room would have protected"
        );
        assert_eq!(
            session.set_admission(Some(AdmissionPolicy::Open), &room, now),
            Err(SessionError::CannotLoosen)
        );
        assert!(
            session.visibility.is_none(),
            "a refused change must not apply"
        );
        assert!(session.admission.is_none());

        // Tightening further is allowed, as is re-inheriting.
        session
            .set_visibility(Some(SessionVisibility::InviteOnly), &room, now)
            .expect("same value is not a loosening");
        session
            .set_admission(Some(AdmissionPolicy::Lobby), &room, now)
            .expect("same value is not a loosening");
    }

    #[test]
    fn clearing_an_override_returns_to_inheritance() {
        let now = OffsetDateTime::now_utc();
        let room = room_with(RoomVisibility::Public, AdmissionPolicy::Open);
        let mut session = Session::schedule(room.id, None, now);
        session
            .set_admission(Some(AdmissionPolicy::Lobby), &room, now)
            .expect("tighten");
        assert_eq!(session.effective_admission(&room), AdmissionPolicy::Lobby);
        session.set_admission(None, &room, now).expect("clear");
        assert_eq!(session.effective_admission(&room), AdmissionPolicy::Open);
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
