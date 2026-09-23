use super::{
    CancelSession, EndSession, GetSession, JoinSession, LeaveSession, ListSessions,
    ScheduleSession, SessionOperationError, StartSession, UpdateSessionDetails, cancel_session,
    end_session, get_session, join_session, leave_session, list_sessions, preflight_join,
    schedule_session, start_session, update_session_details,
};
use crate::ports::{
    AdmissionStore, BlockStore, BoxFuture, ParticipantStore, ParticipationStore, Principal,
    RoomStore, SessionStore, StoreError,
};
use mahfel_domain::{
    ActorId, ActorKind, AdmissionOrigin, AdmissionPolicy, AdmissionRequest, AdmissionRequestId,
    AdmissionRequestStatus, AdmissionTarget, IntervalId, Participant, ParticipantId,
    ParticipantStatus, ParticipationInterval, Room, RoomId, RoomMember, RoomRole, RoomTitle,
    RoomVisibility, Session, SessionError, SessionId, SessionStatus,
};
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;

fn principal(actor_id: ActorId) -> Principal {
    Principal {
        actor_id,
        kind: ActorKind::User,
        display_name: None,
        avatar_url: None,
        email: None,
        email_verified: false,
        username: None,
        roles: Vec::new(),
    }
}

#[derive(Clone, Default)]
struct FakeRoomStore {
    rooms: Arc<Mutex<Vec<Room>>>,
    members: Arc<Mutex<Vec<RoomMember>>>,
}

impl FakeRoomStore {
    fn seed_room(&self, owner: ActorId) -> Room {
        self.seed_room_with_visibility(owner, RoomVisibility::InviteOnly)
    }

    fn seed_room_with_visibility(&self, owner: ActorId, visibility: RoomVisibility) -> Room {
        let now = OffsetDateTime::now_utc();
        let room = Room::create(
            RoomTitle::new("Algebra".to_owned()).expect("valid title"),
            owner,
            visibility,
            now,
        );
        self.rooms.lock().expect("lock").push(room.clone());
        self.members.lock().expect("lock").push(RoomMember::new(
            room.id,
            owner,
            RoomRole::Manager,
            now,
        ));
        room
    }

    fn add_member(&self, room_id: RoomId, actor: ActorId, role: RoomRole) {
        self.members.lock().expect("lock").push(RoomMember::new(
            room_id,
            actor,
            role,
            OffsetDateTime::now_utc(),
        ));
    }
}

impl RoomStore for FakeRoomStore {
    fn get(&self, id: RoomId) -> BoxFuture<'_, Result<Option<Room>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .rooms
                .lock()
                .expect("lock")
                .iter()
                .find(|room| room.id == id)
                .cloned())
        })
    }

    fn insert<'a>(&'a self, room: &'a Room) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            self.rooms.lock().expect("lock").push(room.clone());
            Ok(())
        })
    }

    fn save<'a>(&'a self, room: &'a Room) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut rooms = self.rooms.lock().expect("lock");
            let slot = rooms
                .iter_mut()
                .find(|existing| existing.id == room.id)
                .ok_or_else(|| StoreError::NotFound(room.id.to_string()))?;
            *slot = room.clone();
            Ok(())
        })
    }

    fn create_with_manager<'a>(
        &'a self,
        room: &'a Room,
        manager: &'a RoomMember,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            self.rooms.lock().expect("lock").push(room.clone());
            self.members.lock().expect("lock").push(manager.clone());
            Ok(())
        })
    }

    fn list_for_member(
        &self,
        actor_id: ActorId,
        _limit: u64,
        _offset: u64,
    ) -> BoxFuture<'_, Result<Vec<Room>, StoreError>> {
        Box::pin(async move {
            let rooms = self.rooms.lock().expect("lock");
            let members = self.members.lock().expect("lock");
            Ok(rooms
                .iter()
                .filter(|room| {
                    members
                        .iter()
                        .any(|member| member.room_id == room.id && member.actor_id == actor_id)
                })
                .cloned()
                .collect())
        })
    }

    fn get_member(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<RoomMember>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .members
                .lock()
                .expect("lock")
                .iter()
                .find(|member| member.room_id == room_id && member.actor_id == actor_id)
                .cloned())
        })
    }

    fn list_members(&self, room_id: RoomId) -> BoxFuture<'_, Result<Vec<RoomMember>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .members
                .lock()
                .expect("lock")
                .iter()
                .filter(|member| member.room_id == room_id)
                .cloned()
                .collect())
        })
    }

    fn add_member<'a>(&'a self, member: &'a RoomMember) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            self.members.lock().expect("lock").push(member.clone());
            Ok(())
        })
    }

    fn remove_member(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            let mut members = self.members.lock().expect("lock");
            let before = members.len();
            members.retain(|member| !(member.room_id == room_id && member.actor_id == actor_id));
            Ok(members.len() < before)
        })
    }
}

#[derive(Clone, Default)]
struct FakeAdmissionStore {
    requests: Arc<Mutex<Vec<AdmissionRequest>>>,
}

impl FakeAdmissionStore {
    fn pending(&self, target: AdmissionTarget, actor_id: ActorId) -> Option<AdmissionRequest> {
        self.requests
            .lock()
            .expect("lock")
            .iter()
            .find(|request| {
                request.target == target && request.actor_id == actor_id && request.is_pending()
            })
            .cloned()
    }

    fn all(&self) -> Vec<AdmissionRequest> {
        self.requests.lock().expect("lock").clone()
    }
}

impl AdmissionStore for FakeAdmissionStore {
    fn insert<'a>(
        &'a self,
        request: &'a AdmissionRequest,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut requests = self.requests.lock().expect("lock");
            if requests.iter().any(|existing| {
                existing.target == request.target
                    && existing.actor_id == request.actor_id
                    && existing.is_pending()
            }) {
                return Err(StoreError::Conflict("already pending".to_owned()));
            }
            requests.push(request.clone());
            Ok(())
        })
    }

    fn get_pending(
        &self,
        target: AdmissionTarget,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<AdmissionRequest>, StoreError>> {
        Box::pin(async move { Ok(self.pending(target, actor_id)) })
    }

    fn list_pending_for_room(
        &self,
        room_id: RoomId,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<AdmissionRequest>, StoreError>> {
        Box::pin(async move {
            let mut pending: Vec<AdmissionRequest> = self
                .requests
                .lock()
                .expect("lock")
                .iter()
                .filter(|request| {
                    request.is_pending()
                        && matches!(request.target, AdmissionTarget::Room(id) if id == room_id)
                })
                .cloned()
                .collect();
            pending.sort_by_key(|request| request.requested_at);
            pending.truncate(limit as usize);
            Ok(pending)
        })
    }

    fn get(
        &self,
        id: AdmissionRequestId,
    ) -> BoxFuture<'_, Result<Option<AdmissionRequest>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .requests
                .lock()
                .expect("lock")
                .iter()
                .find(|request| request.id == id)
                .cloned())
        })
    }

    fn list_pending(
        &self,
        session_id: SessionId,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<AdmissionRequest>, StoreError>> {
        Box::pin(async move {
            let mut pending: Vec<AdmissionRequest> = self
                .requests
                .lock()
                .expect("lock")
                .iter()
                .filter(|request| {
                    request.is_pending()
                        && matches!(request.target, AdmissionTarget::Session(id) if id == session_id)
                })
                .cloned()
                .collect();
            pending.sort_by_key(|request| request.requested_at);
            pending.truncate(limit as usize);
            Ok(pending)
        })
    }

    fn count_pending(&self, target: AdmissionTarget) -> BoxFuture<'_, Result<u64, StoreError>> {
        Box::pin(async move {
            Ok(self
                .requests
                .lock()
                .expect("lock")
                .iter()
                .filter(|request| request.target == target && request.is_pending())
                .count() as u64)
        })
    }

    fn decide_if_pending(
        &self,
        id: AdmissionRequestId,
        status: AdmissionRequestStatus,
        decided_by: Option<ActorId>,
        decided_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            let mut requests = self.requests.lock().expect("lock");
            let Some(request) = requests.iter_mut().find(|request| request.id == id) else {
                return Ok(false);
            };
            if !request.is_pending() {
                return Ok(false);
            }
            request.status = status;
            request.decided_at = Some(decided_at);
            request.decided_by = decided_by;
            Ok(true)
        })
    }

    fn expire_pending_for_session(
        &self,
        session_id: SessionId,
        expired_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<u64, StoreError>> {
        Box::pin(async move {
            let mut requests = self.requests.lock().expect("lock");
            let mut expired = 0;
            for request in requests.iter_mut().filter(|request| {
                request.is_pending()
                    && matches!(request.target, AdmissionTarget::Session(id) if id == session_id)
            }) {
                request.status = AdmissionRequestStatus::Expired;
                request.decided_at = Some(expired_at);
                request.decided_by = None;
                expired += 1;
            }
            Ok(expired)
        })
    }
}

#[derive(Clone, Default)]
struct FakeSessionStore {
    sessions: Arc<Mutex<Vec<Session>>>,
}

impl FakeSessionStore {
    fn seed(&self, session: Session) {
        self.sessions.lock().expect("lock").push(session);
    }

    fn status(&self, id: SessionId) -> Option<SessionStatus> {
        self.sessions
            .lock()
            .expect("lock")
            .iter()
            .find(|session| session.id == id)
            .map(|session| session.status)
    }

    fn has_live(&self, room_id: RoomId, except: Option<SessionId>) -> bool {
        self.sessions.lock().expect("lock").iter().any(|session| {
            session.room_id == room_id
                && session.status == SessionStatus::Live
                && except != Some(session.id)
        })
    }
}

impl SessionStore for FakeSessionStore {
    fn get(&self, id: SessionId) -> BoxFuture<'_, Result<Option<Session>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .sessions
                .lock()
                .expect("lock")
                .iter()
                .find(|session| session.id == id)
                .cloned())
        })
    }

    fn insert<'a>(&'a self, session: &'a Session) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            if session.status == SessionStatus::Live && self.has_live(session.room_id, None) {
                return Err(StoreError::Conflict("live session exists".to_owned()));
            }
            self.sessions.lock().expect("lock").push(session.clone());
            Ok(())
        })
    }

    fn save<'a>(&'a self, session: &'a Session) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            if session.status == SessionStatus::Live
                && self.has_live(session.room_id, Some(session.id))
            {
                return Err(StoreError::Conflict("live session exists".to_owned()));
            }
            let mut sessions = self.sessions.lock().expect("lock");
            let slot = sessions
                .iter_mut()
                .find(|existing| existing.id == session.id)
                .ok_or_else(|| StoreError::NotFound(session.id.to_string()))?;
            *slot = session.clone();
            Ok(())
        })
    }

    fn save_details<'a>(
        &'a self,
        session: &'a Session,
        fields: crate::ports::SessionDetailsFields,
    ) -> BoxFuture<'a, Result<Session, StoreError>> {
        Box::pin(async move {
            let mut sessions = self.sessions.lock().expect("lock");
            let slot = sessions
                .iter_mut()
                .find(|existing| existing.id == session.id)
                .ok_or_else(|| StoreError::NotFound(session.id.to_string()))?;
            if fields.title {
                slot.title = session.title.clone();
            }
            if fields.scheduled_start_at {
                slot.scheduled_start_at = session.scheduled_start_at;
            }
            if fields.scheduled_duration_minutes {
                slot.scheduled_duration_minutes = session.scheduled_duration_minutes;
            }
            slot.updated_at = session.updated_at;
            Ok(slot.clone())
        })
    }

    fn list_for_room(&self, room_id: RoomId) -> BoxFuture<'_, Result<Vec<Session>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .sessions
                .lock()
                .expect("lock")
                .iter()
                .filter(|session| session.room_id == room_id)
                .cloned()
                .collect())
        })
    }
}

fn actor() -> ActorId {
    ActorId::new()
}

fn scheduled(room_id: RoomId) -> Session {
    Session::schedule(room_id, None, OffsetDateTime::now_utc())
}

/// In-memory [`ParticipationStore`] mirroring the two database invariants the
/// use cases rely on: at most one open interval per participant, and a
/// correction that retains the superseded row.
#[derive(Clone, Default)]
pub(crate) struct FakeParticipationStore {
    intervals: Arc<Mutex<Vec<ParticipationInterval>>>,
}

impl FakeParticipationStore {
    fn intervals_for(&self, participant_id: ParticipantId) -> Vec<ParticipationInterval> {
        self.intervals
            .lock()
            .expect("lock")
            .iter()
            .filter(|interval| interval.participant_id == participant_id)
            .cloned()
            .collect()
    }
}

impl ParticipationStore for FakeParticipationStore {
    fn open_interval<'a>(
        &'a self,
        interval: &'a ParticipationInterval,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut intervals = self.intervals.lock().expect("lock");
            if intervals.iter().any(|existing| {
                existing.participant_id == interval.participant_id && existing.is_open()
            }) {
                return Err(StoreError::Conflict("already open".to_owned()));
            }
            intervals.push(interval.clone());
            Ok(())
        })
    }

    fn close_open_interval(
        &self,
        participant_id: ParticipantId,
        ended_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<Option<IntervalId>, StoreError>> {
        Box::pin(async move {
            let mut intervals = self.intervals.lock().expect("lock");
            match intervals
                .iter_mut()
                .find(|interval| interval.participant_id == participant_id && interval.is_open())
            {
                Some(interval) => {
                    interval.ended_at = Some(ended_at);
                    Ok(Some(interval.id))
                }
                None => Ok(None),
            }
        })
    }

    fn close_open_intervals_for_session(
        &self,
        session_id: SessionId,
        ended_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<u64, StoreError>> {
        Box::pin(async move {
            let mut intervals = self.intervals.lock().expect("lock");
            let mut closed = 0_u64;
            for interval in intervals.iter_mut() {
                if interval.session_id == session_id && interval.is_open() {
                    interval.ended_at = Some(ended_at);
                    closed += 1;
                }
            }
            Ok(closed)
        })
    }

    fn get(
        &self,
        id: IntervalId,
    ) -> BoxFuture<'_, Result<Option<ParticipationInterval>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .intervals
                .lock()
                .expect("lock")
                .iter()
                .find(|interval| interval.id == id)
                .cloned())
        })
    }

    fn list_for_participant(
        &self,
        participant_id: ParticipantId,
    ) -> BoxFuture<'_, Result<Vec<ParticipationInterval>, StoreError>> {
        Box::pin(async move { Ok(self.intervals_for(participant_id)) })
    }

    fn list_for_session(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Vec<ParticipationInterval>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .intervals
                .lock()
                .expect("lock")
                .iter()
                .filter(|interval| interval.session_id == session_id)
                .cloned()
                .collect())
        })
    }

    fn supersede_interval<'a>(
        &'a self,
        original: &'a ParticipationInterval,
        replacement: &'a ParticipationInterval,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut intervals = self.intervals.lock().expect("lock");
            let slot = intervals
                .iter_mut()
                .find(|interval| interval.id == original.id)
                .ok_or_else(|| StoreError::NotFound(original.id.to_string()))?;
            if slot.is_superseded() {
                return Err(StoreError::Conflict("already superseded".to_owned()));
            }
            *slot = original.clone();
            intervals.push(replacement.clone());
            Ok(())
        })
    }
}

/// In-memory [`ParticipantStore`] mirroring the partial unique index that keeps
/// one active participant per `(session, actor)`.
#[derive(Clone, Default)]
pub(crate) struct FakeParticipantStore {
    participants: Arc<Mutex<Vec<Participant>>>,
}

impl FakeParticipantStore {
    fn active(&self, session_id: SessionId, actor_id: ActorId) -> Option<Participant> {
        self.participants
            .lock()
            .expect("lock")
            .iter()
            .find(|p| p.session_id == session_id && p.actor_id == actor_id && p.is_active())
            .cloned()
    }
}

impl ParticipantStore for FakeParticipantStore {
    fn get(&self, id: ParticipantId) -> BoxFuture<'_, Result<Option<Participant>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .participants
                .lock()
                .expect("lock")
                .iter()
                .find(|p| p.id == id)
                .cloned())
        })
    }

    fn get_active(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<Participant>, StoreError>> {
        Box::pin(async move { Ok(self.active(session_id, actor_id)) })
    }

    fn insert<'a>(&'a self, participant: &'a Participant) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            if participant.is_active()
                && self
                    .active(participant.session_id, participant.actor_id)
                    .is_some()
            {
                return Err(StoreError::Conflict("already active".to_owned()));
            }
            self.participants
                .lock()
                .expect("lock")
                .push(participant.clone());
            Ok(())
        })
    }

    fn save<'a>(&'a self, participant: &'a Participant) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut participants = self.participants.lock().expect("lock");
            let slot = participants
                .iter_mut()
                .find(|existing| existing.id == participant.id)
                .ok_or_else(|| StoreError::NotFound(participant.id.to_string()))?;
            *slot = participant.clone();
            Ok(())
        })
    }

    fn list_for_session(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Vec<Participant>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .participants
                .lock()
                .expect("lock")
                .iter()
                .filter(|p| p.session_id == session_id)
                .cloned()
                .collect())
        })
    }
}

/// Blocks are absent unless a test seeds one, so the join tests that predate
/// them exercise the same path as before.
#[derive(Clone, Default)]
pub(crate) struct FakeBlockStore {
    pub(crate) blocks: Arc<Mutex<Vec<mahfel_domain::Block>>>,
}

impl BlockStore for FakeBlockStore {
    fn get_active(
        &self,
        scope: mahfel_domain::BlockScope,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<mahfel_domain::Block>, StoreError>> {
        Box::pin(async move {
            let now = OffsetDateTime::now_utc();
            Ok(self
                .blocks
                .lock()
                .expect("lock")
                .iter()
                .find(|b| b.scope == scope && b.actor_id == actor_id && b.is_active_at(now))
                .cloned())
        })
    }

    fn insert<'a>(
        &'a self,
        block: &'a mahfel_domain::Block,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            self.blocks.lock().expect("lock").push(block.clone());
            Ok(())
        })
    }

    fn extend(
        &self,
        id: mahfel_domain::BlockId,
        expires_at: Option<OffsetDateTime>,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            let mut blocks = self.blocks.lock().expect("lock");
            let Some(block) = blocks
                .iter_mut()
                .find(|b| b.id == id && b.lifted_at.is_none())
            else {
                return Ok(false);
            };
            block.expires_at = expires_at;
            Ok(true)
        })
    }

    fn lift(
        &self,
        id: mahfel_domain::BlockId,
        lifted_by: ActorId,
        lifted_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            let mut blocks = self.blocks.lock().expect("lock");
            let Some(block) = blocks
                .iter_mut()
                .find(|b| b.id == id && b.lifted_at.is_none())
            else {
                return Ok(false);
            };
            block.lifted_at = Some(lifted_at);
            block.lifted_by = Some(lifted_by);
            Ok(true)
        })
    }

    fn list_active_for_room(
        &self,
        room_id: mahfel_domain::RoomId,
        _limit: u32,
    ) -> BoxFuture<'_, Result<Vec<mahfel_domain::Block>, StoreError>> {
        Box::pin(async move {
            let now = OffsetDateTime::now_utc();
            Ok(self
                .blocks
                .lock()
                .expect("lock")
                .iter()
                .filter(|b| b.scope.room_id() == Some(room_id) && b.is_active_at(now))
                .cloned()
                .collect())
        })
    }

    fn list_active_for_session(
        &self,
        session_id: SessionId,
        _limit: u32,
    ) -> BoxFuture<'_, Result<Vec<mahfel_domain::Block>, StoreError>> {
        Box::pin(async move {
            let now = OffsetDateTime::now_utc();
            Ok(self
                .blocks
                .lock()
                .expect("lock")
                .iter()
                .filter(|b| b.scope.session_id() == Some(session_id) && b.is_active_at(now))
                .cloned()
                .collect())
        })
    }
}

/// A room with one manager, and a live session inside it.
fn live_fixture() -> (
    FakeRoomStore,
    FakeSessionStore,
    FakeParticipantStore,
    FakeParticipationStore,
    Room,
    Session,
    ActorId,
) {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let participants = FakeParticipantStore::default();
    let participation = FakeParticipationStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);
    let mut session = scheduled(room.id);
    session.start(OffsetDateTime::now_utc()).expect("start");
    sessions.seed(session.clone());
    (
        rooms,
        sessions,
        participants,
        participation,
        room,
        session,
        owner,
    )
}

#[tokio::test]
async fn reconnect_reuses_the_same_participant() {
    let (rooms, sessions, participants, participation, _room, session, owner) = live_fixture();

    let admissions = FakeAdmissionStore::default();
    let first = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: owner,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("first join")
    .participant()
    .expect("admitted immediately");

    let second = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: owner,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("reconnect")
    .participant()
    .expect("admitted immediately");

    assert_eq!(first.id, second.id, "reconnect must reuse the identity");
    assert_eq!(
        participants
            .list_for_session(session.id)
            .await
            .expect("list")
            .len(),
        1,
        "a reconnect must not duplicate the participant row"
    );
}

#[tokio::test]
async fn join_is_deny_by_default_for_non_members_of_private_rooms() {
    let admissions = FakeAdmissionStore::default();
    let (rooms, sessions, participants, participation, _room, session, owner) = live_fixture();
    let _ = owner;
    let outsider = actor();

    let denied = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: outsider,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await;
    assert!(matches!(
        denied,
        Err(super::SessionOperationError::Forbidden)
    ));
}

#[tokio::test]
async fn non_members_may_join_public_rooms() {
    let admissions = FakeAdmissionStore::default();
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let participants = FakeParticipantStore::default();
    let participation = FakeParticipationStore::default();
    let owner = actor();
    let room = rooms.seed_room_with_visibility(owner, RoomVisibility::Public);
    let mut session = scheduled(room.id);
    session.start(OffsetDateTime::now_utc()).expect("start");
    sessions.seed(session.clone());

    let joined = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: actor(),
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("public join")
    .participant()
    .expect("admitted immediately");
    assert!(joined.is_active());
}

#[tokio::test]
async fn join_requires_a_live_session() {
    let admissions = FakeAdmissionStore::default();
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let participants = FakeParticipantStore::default();
    let participation = FakeParticipationStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);
    let session = scheduled(room.id);
    sessions.seed(session.clone());

    let denied = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: owner,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await;
    assert!(matches!(denied, Err(super::SessionOperationError::NotLive)));
}

#[tokio::test]
async fn leave_is_terminal_and_the_actor_can_rejoin() {
    let admissions = FakeAdmissionStore::default();
    let (rooms, sessions, participants, participation, _room, session, owner) = live_fixture();
    let joined = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: owner,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("join")
    .participant()
    .expect("admitted immediately");

    leave_session(
        &participants,
        &participation,
        LeaveSession {
            actor: owner,
            participant_id: joined.id,
        },
    )
    .await
    .expect("leave");
    assert_eq!(
        participants
            .get(joined.id)
            .await
            .expect("get")
            .expect("present")
            .status,
        ParticipantStatus::Left
    );

    let rejoined = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: owner,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("rejoin")
    .participant()
    .expect("admitted immediately");
    assert_ne!(rejoined.id, joined.id, "a rejoin is a new participation");
    assert!(rejoined.is_active());
}

#[tokio::test]
async fn leaving_another_actors_participant_is_not_found() {
    let admissions = FakeAdmissionStore::default();
    let (rooms, sessions, participants, participation, _room, session, owner) = live_fixture();
    let joined = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: owner,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("join")
    .participant()
    .expect("admitted immediately");

    let denied = leave_session(
        &participants,
        &participation,
        LeaveSession {
            actor: actor(),
            participant_id: joined.id,
        },
    )
    .await;
    assert!(matches!(
        denied,
        Err(super::SessionOperationError::NotFound)
    ));
}

#[tokio::test]
async fn non_manager_is_denied_and_non_member_is_hidden() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);
    let member = actor();
    rooms.add_member(room.id, member, RoomRole::Member);
    let stranger = actor();

    let denied = schedule_session(
        &sessions,
        &rooms,
        ScheduleSession {
            principal: principal(member),
            room_id: room.id,
            title: None,
            scheduled_start_at: None,
            scheduled_duration_minutes: None,
        },
    )
    .await;
    assert!(
        matches!(denied, Err(super::SessionOperationError::Forbidden)),
        "a member without the manager role is forbidden, got {denied:?}"
    );

    let hidden = schedule_session(
        &sessions,
        &rooms,
        ScheduleSession {
            principal: principal(stranger),
            room_id: room.id,
            title: None,
            scheduled_start_at: None,
            scheduled_duration_minutes: None,
        },
    )
    .await;
    assert!(
        matches!(hidden, Err(super::SessionOperationError::NotFound)),
        "a non-member must not learn the room exists, got {hidden:?}"
    );
}

#[tokio::test]
async fn session_details_are_validated_and_editable_in_every_state() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);
    let member = actor();
    rooms.add_member(room.id, member, RoomRole::Member);

    let invalid = schedule_session(
        &sessions,
        &rooms,
        ScheduleSession {
            principal: principal(owner),
            room_id: room.id,
            title: Some("  ".to_owned()),
            scheduled_start_at: None,
            scheduled_duration_minutes: Some(0),
        },
    )
    .await;
    assert!(matches!(invalid, Err(SessionOperationError::Validation(_))));

    for status in [
        SessionStatus::Scheduled,
        SessionStatus::Live,
        SessionStatus::Ended,
        SessionStatus::Cancelled,
    ] {
        let mut session = scheduled(room.id);
        let now = OffsetDateTime::now_utc();
        match status {
            SessionStatus::Scheduled => {}
            SessionStatus::Live => session.start(now).expect("start"),
            SessionStatus::Ended => {
                session.start(now).expect("start");
                session.end(now).expect("end");
            }
            SessionStatus::Cancelled => session.cancel(now).expect("cancel"),
        }
        sessions.seed(session.clone());

        let denied = update_session_details(
            &sessions,
            &rooms,
            UpdateSessionDetails {
                principal: principal(member),
                session_id: session.id,
                title: Some(Some("Denied".to_owned())),
                scheduled_start_at: None,
                scheduled_duration_minutes: None,
            },
        )
        .await;
        assert!(matches!(denied, Err(SessionOperationError::Forbidden)));

        let updated = update_session_details(
            &sessions,
            &rooms,
            UpdateSessionDetails {
                principal: principal(owner),
                session_id: session.id,
                title: Some(Some("  Custom  ".to_owned())),
                scheduled_start_at: None,
                scheduled_duration_minutes: Some(Some(30)),
            },
        )
        .await
        .expect("update details");
        assert_eq!(updated.status, status);
        assert_eq!(
            updated.title.as_ref().map(|value| value.as_str()),
            Some("Custom")
        );
        assert_eq!(
            updated.scheduled_duration_minutes.map(|value| value.get()),
            Some(30)
        );

        let cleared = update_session_details(
            &sessions,
            &rooms,
            UpdateSessionDetails {
                principal: principal(owner),
                session_id: session.id,
                title: Some(None),
                scheduled_start_at: None,
                scheduled_duration_minutes: Some(None),
            },
        )
        .await
        .expect("clear details");
        assert!(cleared.title.is_none());
        assert!(cleared.scheduled_duration_minutes.is_none());
        assert_eq!(cleared.status, status);
    }
}

#[tokio::test]
async fn capable_admin_can_start_before_the_planned_time() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let room = rooms.seed_room(actor());
    let mut admin = principal(actor());
    admin.roles.push("admin".to_owned());
    let future = OffsetDateTime::now_utc() + time::Duration::days(2);
    let scheduled = schedule_session(
        &sessions,
        &rooms,
        ScheduleSession {
            principal: admin.clone(),
            room_id: room.id,
            title: None,
            scheduled_start_at: Some(future),
            scheduled_duration_minutes: Some(60),
        },
    )
    .await
    .expect("admin has manage_session");
    let live = start_session(
        &sessions,
        &rooms,
        StartSession {
            principal: admin,
            session_id: scheduled.id,
        },
    )
    .await
    .expect("planned start is advisory");
    assert_eq!(live.status, SessionStatus::Live);
    assert!(live.started_at.expect("start time") < future);
}

#[tokio::test]
async fn invalid_transition_is_rejected_without_state_change() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let participation = FakeParticipationStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);

    let session = scheduled(room.id);
    sessions.seed(session.clone());

    // Ending a scheduled session is invalid.
    let ended = end_session(
        &sessions,
        &rooms,
        &participation,
        &FakeAdmissionStore::default(),
        EndSession {
            actor: owner,
            session_id: session.id,
        },
    )
    .await;
    assert!(
        matches!(
            ended,
            Err(super::SessionOperationError::Domain(
                SessionError::CannotEnd(_)
            ))
        ),
        "end from scheduled must be a domain conflict, got {ended:?}"
    );
    assert_eq!(
        sessions.status(session.id),
        Some(SessionStatus::Scheduled),
        "a rejected transition must not persist"
    );

    // Cancelling a live session is invalid.
    let mut live = scheduled(room.id);
    live.start(OffsetDateTime::now_utc()).expect("start");
    sessions.seed(live.clone());
    let cancelled = cancel_session(
        &sessions,
        &rooms,
        CancelSession {
            actor: owner,
            session_id: live.id,
        },
    )
    .await;
    assert!(
        matches!(
            cancelled,
            Err(super::SessionOperationError::Domain(
                SessionError::CannotCancel(_)
            ))
        ),
        "cancel from live must be a domain conflict, got {cancelled:?}"
    );
    assert_eq!(sessions.status(live.id), Some(SessionStatus::Live));
}

#[tokio::test]
async fn manager_can_schedule_start_end_and_cancel() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let participation = FakeParticipationStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);

    let scheduled_session = schedule_session(
        &sessions,
        &rooms,
        ScheduleSession {
            principal: principal(owner),
            room_id: room.id,
            title: None,
            scheduled_start_at: None,
            scheduled_duration_minutes: None,
        },
    )
    .await
    .expect("schedule");
    assert_eq!(scheduled_session.status, SessionStatus::Scheduled);

    let live = start_session(
        &sessions,
        &rooms,
        StartSession {
            principal: principal(owner),
            session_id: scheduled_session.id,
        },
    )
    .await
    .expect("start");
    assert_eq!(live.status, SessionStatus::Live);

    let ended = end_session(
        &sessions,
        &rooms,
        &participation,
        &FakeAdmissionStore::default(),
        EndSession {
            actor: owner,
            session_id: live.id,
        },
    )
    .await
    .expect("end");
    assert_eq!(ended.status, SessionStatus::Ended);

    let to_cancel = scheduled(room.id);
    sessions.seed(to_cancel.clone());
    let cancelled = cancel_session(
        &sessions,
        &rooms,
        CancelSession {
            actor: owner,
            session_id: to_cancel.id,
        },
    )
    .await
    .expect("cancel");
    assert_eq!(cancelled.status, SessionStatus::Cancelled);
}

#[tokio::test]
async fn second_live_session_conflicts() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);

    let first = scheduled(room.id);
    let second = scheduled(room.id);
    sessions.seed(first.clone());
    sessions.seed(second.clone());

    start_session(
        &sessions,
        &rooms,
        StartSession {
            principal: principal(owner),
            session_id: first.id,
        },
    )
    .await
    .expect("first start wins");

    let losing = start_session(
        &sessions,
        &rooms,
        StartSession {
            principal: principal(owner),
            session_id: second.id,
        },
    )
    .await;
    assert!(
        matches!(losing, Err(super::SessionOperationError::Conflict)),
        "the second live start must conflict, got {losing:?}"
    );
    assert_eq!(sessions.status(second.id), Some(SessionStatus::Scheduled));
}

#[tokio::test]
async fn manager_can_get_and_list_room_sessions() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);
    let session = scheduled(room.id);
    sessions.seed(session.clone());

    let listed = list_sessions(
        &sessions,
        &rooms,
        ListSessions {
            actor: owner,
            room_id: room.id,
        },
    )
    .await
    .expect("list");
    assert_eq!(listed, vec![session.clone()]);

    let loaded = get_session(
        &sessions,
        &rooms,
        GetSession {
            actor: owner,
            session_id: session.id,
        },
    )
    .await
    .expect("get");
    assert_eq!(loaded, session);
}

#[tokio::test]
async fn a_member_may_load_a_session_it_may_join() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);
    let member = actor();
    rooms.add_member(room.id, member, RoomRole::Member);
    let session = scheduled(room.id);
    sessions.seed(session.clone());

    let loaded = super::get_session_for_caller(
        &sessions,
        &rooms,
        super::GetSessionForCaller {
            actor: member,
            session_id: session.id,
        },
    )
    .await
    .expect("a member may read a session it may join");
    assert_eq!(loaded, session);
}

#[tokio::test]
async fn an_outsider_cannot_load_a_private_session() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);
    let session = scheduled(room.id);
    sessions.seed(session.clone());

    let denied = super::get_session_for_caller(
        &sessions,
        &rooms,
        super::GetSessionForCaller {
            actor: actor(),
            session_id: session.id,
        },
    )
    .await;
    assert!(
        matches!(denied, Err(super::SessionOperationError::NotFound)),
        "an outsider must not learn the session exists, got {denied:?}"
    );
}

/// A live fixture whose room requires approval, so a join waits.
async fn lobby_fixture() -> (
    FakeRoomStore,
    FakeSessionStore,
    FakeAdmissionStore,
    FakeParticipantStore,
    FakeParticipationStore,
    Room,
    Session,
    ActorId,
) {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let admissions = FakeAdmissionStore::default();
    let participants = FakeParticipantStore::default();
    let participation = FakeParticipationStore::default();
    let owner = actor();
    // Public, so a synthetic actor passes authorization and reaches the lobby.
    // Admission is what these tests vary, not visibility.
    let mut room = rooms.seed_room_with_visibility(owner, RoomVisibility::Public);
    room.set_admission(AdmissionPolicy::Lobby, OffsetDateTime::now_utc())
        .expect("lobby");
    rooms.save(&room).await.expect("save");
    let mut session = scheduled(room.id);
    session.start(OffsetDateTime::now_utc()).expect("start");
    sessions.seed(session.clone());
    (
        rooms,
        sessions,
        admissions,
        participants,
        participation,
        room,
        session,
        owner,
    )
}

#[tokio::test]
async fn lobby_defers_the_join_and_grants_no_participant() {
    let (rooms, sessions, admissions, participants, participation, _room, session, owner) =
        lobby_fixture().await;

    let outcome = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: owner,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("authorized");

    assert!(
        outcome.is_waiting(),
        "an authorized caller waits under lobby"
    );
    assert!(
        outcome.participant().is_none(),
        "a waiting caller must get no participant"
    );
    assert!(
        participants
            .list_for_session(session.id)
            .await
            .expect("list")
            .is_empty(),
        "a waiting caller must not appear in the roster"
    );
    assert!(
        admissions
            .pending(AdmissionTarget::Session(session.id), owner)
            .is_some(),
        "the request must be queued"
    );
}

#[tokio::test]
async fn an_approved_participant_can_reconnect_under_lobby_policy() {
    let (rooms, sessions, admissions, participants, participation, room, session, owner) =
        lobby_fixture().await;
    let approved = Participant::join(session.id, owner, OffsetDateTime::now_utc());
    participants
        .insert(&approved)
        .await
        .expect("approved participant");
    let command = JoinSession {
        actor: owner,
        session_id: session.id,
        origin: AdmissionOrigin::Join,
    };
    assert_eq!(
        preflight_join(
            &participants,
            &sessions,
            &rooms,
            &FakeBlockStore::default(),
            &command,
        )
        .await
        .expect("preflight"),
        Some(room.id),
    );
    let joined = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        command,
    )
    .await
    .expect("reconnect")
    .participant()
    .expect("joined");
    assert_eq!(joined.id, approved.id);
    assert!(
        admissions.all().is_empty(),
        "approval must not queue a second request"
    );
}

#[tokio::test]
async fn open_policy_still_admits_immediately() {
    let (rooms, sessions, participants, participation, _room, session, owner) = live_fixture();
    let admissions = FakeAdmissionStore::default();

    let outcome = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: owner,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("join");

    assert!(
        !outcome.is_waiting(),
        "the default policy admits immediately"
    );
    assert!(
        admissions.all().is_empty(),
        "no request is queued under open"
    );
}

#[tokio::test]
async fn a_repeated_join_while_waiting_reuses_the_same_request() {
    let (rooms, sessions, admissions, participants, participation, _room, session, owner) =
        lobby_fixture().await;

    let mut request_ids = Vec::new();
    for _ in 0..3 {
        let outcome = join_session(
            &participants,
            &participation,
            &admissions,
            &sessions,
            &rooms,
            &FakeBlockStore::default(),
            JoinSession {
                actor: owner,
                session_id: session.id,
                origin: AdmissionOrigin::Join,
            },
        )
        .await
        .expect("authorized");
        match outcome {
            super::JoinOutcome::Waiting(request) => request_ids.push(request.id),
            super::JoinOutcome::Joined { .. } => panic!("lobby must not admit directly"),
        }
    }

    assert_eq!(request_ids.len(), 3);
    assert_eq!(
        request_ids[0], request_ids[1],
        "a repeat join must keep its place rather than queue again"
    );
    assert_eq!(request_ids[1], request_ids[2]);
    assert_eq!(
        admissions.all().len(),
        1,
        "the waiting set must not grow per attempt"
    );
}

#[tokio::test]
async fn the_lobby_is_bounded() {
    let (rooms, sessions, admissions, participants, participation, _room, session, _owner) =
        lobby_fixture().await;

    for _ in 0..mahfel_domain::MAX_PENDING_PER_SESSION {
        // A distinct actor each time, so each consumes a slot.
        let outcome = join_session(
            &participants,
            &participation,
            &admissions,
            &sessions,
            &rooms,
            &FakeBlockStore::default(),
            JoinSession {
                actor: actor(),
                session_id: session.id,
                origin: AdmissionOrigin::Join,
            },
        )
        .await;
        assert!(outcome.is_ok(), "within the cap a request is queued");
    }

    let overflow = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: actor(),
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await;
    assert!(
        overflow.is_err(),
        "a full lobby must refuse rather than grow without bound"
    );
}

#[tokio::test]
async fn admitting_creates_the_participant_and_opens_an_interval() {
    let (rooms, sessions, admissions, participants, participation, room, session, owner) =
        lobby_fixture().await;
    let waiter = actor();
    rooms.add_member(room.id, waiter, RoomRole::Member);

    join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: waiter,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("queued");

    let grant = crate::admission::admit(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        crate::admission::AdmissionDecision {
            actor: owner,
            target: AdmissionTarget::Session(session.id),
            subject: waiter,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("admit");

    let participant = grant
        .participant()
        .expect("a session approval grants a participant");
    assert_eq!(participant.actor_id, waiter);
    assert!(
        participants
            .get_active(session.id, waiter)
            .await
            .expect("get")
            .is_some()
    );
    assert!(
        !participation
            .list_for_participant(participant.id)
            .await
            .expect("intervals")
            .is_empty(),
        "an approved join must start recording connected time"
    );
    assert!(
        admissions
            .pending(AdmissionTarget::Session(session.id), waiter)
            .is_none(),
        "the request must no longer be pending"
    );
}

#[tokio::test]
async fn rejecting_grants_nothing_and_is_terminal() {
    let (rooms, sessions, admissions, participants, participation, room, session, owner) =
        lobby_fixture().await;
    let waiter = actor();
    rooms.add_member(room.id, waiter, RoomRole::Member);

    join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: waiter,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("queued");

    crate::admission::reject(
        &admissions,
        &sessions,
        &rooms,
        crate::admission::AdmissionDecision {
            actor: owner,
            target: AdmissionTarget::Session(session.id),
            subject: waiter,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("reject");

    assert!(
        participants
            .get_active(session.id, waiter)
            .await
            .expect("get")
            .is_none(),
        "a rejected caller must not become a participant"
    );
    let again = crate::admission::reject(
        &admissions,
        &sessions,
        &rooms,
        crate::admission::AdmissionDecision {
            actor: owner,
            target: AdmissionTarget::Session(session.id),
            subject: waiter,
        },
        OffsetDateTime::now_utc(),
    )
    .await;
    assert!(again.is_err(), "rejection is terminal, not repeatable");
}

#[tokio::test]
async fn a_member_who_cannot_admit_is_denied() {
    let (rooms, sessions, admissions, participants, participation, room, session, _owner) =
        lobby_fixture().await;
    let member = actor();
    rooms.add_member(room.id, member, RoomRole::Member);
    let waiter = actor();
    rooms.add_member(room.id, waiter, RoomRole::Member);

    join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: waiter,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("queued");

    let denied = crate::admission::admit(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        crate::admission::AdmissionDecision {
            actor: member,
            target: AdmissionTarget::Session(session.id),
            subject: waiter,
        },
        OffsetDateTime::now_utc(),
    )
    .await;
    assert!(
        denied.is_err(),
        "a plain member must not admit, even though it may see the session"
    );
}

#[tokio::test]
async fn ending_a_session_retires_the_lobby() {
    let (rooms, sessions, admissions, participants, participation, room, session, owner) =
        lobby_fixture().await;
    let waiter = actor();
    rooms.add_member(room.id, waiter, RoomRole::Member);

    join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &FakeBlockStore::default(),
        JoinSession {
            actor: waiter,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("queued");

    end_session(
        &sessions,
        &rooms,
        &participation,
        &admissions,
        EndSession {
            actor: owner,
            session_id: session.id,
        },
    )
    .await
    .expect("end");

    assert!(
        admissions.all().iter().all(|request| !request.is_pending()),
        "no one may be left waiting on a session that ended"
    );
    assert_eq!(
        sessions.status(session.id),
        Some(SessionStatus::Ended),
        "the session itself still ends"
    );
}

/// Seed one active block, so a join test can prove the guard reads it.
fn seeded_blocks(scope: mahfel_domain::BlockScope, subject: ActorId) -> FakeBlockStore {
    let store = FakeBlockStore::default();
    let block = mahfel_domain::Block::new(
        scope,
        subject,
        ActorId::new(),
        None,
        None,
        OffsetDateTime::now_utc(),
    )
    .expect("block");
    store.blocks.lock().expect("lock").push(block);
    store
}

#[tokio::test]
async fn a_room_block_denies_a_join_to_any_session_in_that_room() {
    let (rooms, sessions, participants, participation, room, session, _owner) = live_fixture();
    let blocked = ActorId::new();
    // No membership and an invite-only room, so the block is what denies this,
    // not visibility.
    let blocks = seeded_blocks(mahfel_domain::BlockScope::Room(room.id), blocked);

    let result = join_session(
        &participants,
        &participation,
        &FakeAdmissionStore::default(),
        &sessions,
        &rooms,
        &blocks,
        JoinSession {
            actor: blocked,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await;

    assert!(
        matches!(result, Err(SessionOperationError::Forbidden)),
        "a room block denies the session without a per-session row, got {result:?}"
    );
}

#[tokio::test]
async fn a_session_block_denies_that_session_even_for_a_member() {
    let (rooms, sessions, participants, participation, _room, session, owner) = live_fixture();
    let blocks = seeded_blocks(mahfel_domain::BlockScope::Session(session.id), owner);

    let result = join_session(
        &participants,
        &participation,
        &FakeAdmissionStore::default(),
        &sessions,
        &rooms,
        &blocks,
        JoinSession {
            actor: owner,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await;

    assert!(
        matches!(result, Err(SessionOperationError::Forbidden)),
        "a session block denies a narrow case the room would allow, got {result:?}"
    );
}

#[tokio::test]
async fn a_block_on_another_session_does_not_deny_this_one() {
    let (rooms, sessions, participants, participation, _room, session, owner) = live_fixture();
    let blocks = seeded_blocks(mahfel_domain::BlockScope::Session(SessionId::new()), owner);

    let joined = join_session(
        &participants,
        &participation,
        &FakeAdmissionStore::default(),
        &sessions,
        &rooms,
        &blocks,
        JoinSession {
            actor: owner,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await
    .expect("a block elsewhere must not deny this join");

    assert!(joined.participant().is_some());
}

#[tokio::test]
async fn a_blocked_principal_does_not_occupy_a_lobby_slot() {
    // The block is checked before admission, so a blocked principal never
    // queues and cannot consume lobby capacity.
    let (rooms, sessions, admissions, participants, participation, room, session, _owner) =
        lobby_fixture().await;
    let blocked = ActorId::new();
    let blocks = seeded_blocks(mahfel_domain::BlockScope::Room(room.id), blocked);

    let result = join_session(
        &participants,
        &participation,
        &admissions,
        &sessions,
        &rooms,
        &blocks,
        JoinSession {
            actor: blocked,
            session_id: session.id,
            origin: AdmissionOrigin::Join,
        },
    )
    .await;

    assert!(matches!(result, Err(SessionOperationError::Forbidden)));
    assert!(
        admissions.requests.lock().expect("lock").is_empty(),
        "a blocked principal must not reach the lobby"
    );
}
