use super::{
    CancelSession, EndSession, GetSession, JoinSession, LeaveSession, ListSessions,
    ScheduleSession, StartSession, cancel_session, end_session, get_session, join_session,
    leave_session, list_sessions, schedule_session, start_session,
};
use crate::ports::{BoxFuture, ParticipantStore, RoomStore, SessionStore, StoreError};
use mahfel_domain::{
    ActorId, Participant, ParticipantId, ParticipantStatus, Room, RoomId, RoomMember, RoomRole,
    RoomTitle, RoomVisibility, Session, SessionError, SessionId, SessionStatus,
};
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;

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

/// In-memory [`ParticipantStore`] mirroring the partial unique index that keeps
/// one active participant per `(session, actor)`.
#[derive(Clone, Default)]
struct FakeParticipantStore {
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

/// A room with one manager, and a live session inside it.
fn live_fixture() -> (
    FakeRoomStore,
    FakeSessionStore,
    FakeParticipantStore,
    Room,
    Session,
    ActorId,
) {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let participants = FakeParticipantStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);
    let mut session = scheduled(room.id);
    session.start(OffsetDateTime::now_utc()).expect("start");
    sessions.seed(session.clone());
    (rooms, sessions, participants, room, session, owner)
}

#[tokio::test]
async fn reconnect_reuses_the_same_participant() {
    let (rooms, sessions, participants, _room, session, owner) = live_fixture();

    let first = join_session(
        &participants,
        &sessions,
        &rooms,
        JoinSession {
            actor: owner,
            session_id: session.id,
        },
    )
    .await
    .expect("first join");

    let second = join_session(
        &participants,
        &sessions,
        &rooms,
        JoinSession {
            actor: owner,
            session_id: session.id,
        },
    )
    .await
    .expect("reconnect");

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
    let (rooms, sessions, participants, _room, session, owner) = live_fixture();
    let _ = owner;
    let outsider = actor();

    let denied = join_session(
        &participants,
        &sessions,
        &rooms,
        JoinSession {
            actor: outsider,
            session_id: session.id,
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
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let participants = FakeParticipantStore::default();
    let owner = actor();
    let room = rooms.seed_room_with_visibility(owner, RoomVisibility::Public);
    let mut session = scheduled(room.id);
    session.start(OffsetDateTime::now_utc()).expect("start");
    sessions.seed(session.clone());

    let joined = join_session(
        &participants,
        &sessions,
        &rooms,
        JoinSession {
            actor: actor(),
            session_id: session.id,
        },
    )
    .await
    .expect("public join");
    assert!(joined.is_active());
}

#[tokio::test]
async fn join_requires_a_live_session() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let participants = FakeParticipantStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);
    let session = scheduled(room.id);
    sessions.seed(session.clone());

    let denied = join_session(
        &participants,
        &sessions,
        &rooms,
        JoinSession {
            actor: owner,
            session_id: session.id,
        },
    )
    .await;
    assert!(matches!(denied, Err(super::SessionOperationError::NotLive)));
}

#[tokio::test]
async fn leave_is_terminal_and_the_actor_can_rejoin() {
    let (rooms, sessions, participants, _room, session, owner) = live_fixture();
    let joined = join_session(
        &participants,
        &sessions,
        &rooms,
        JoinSession {
            actor: owner,
            session_id: session.id,
        },
    )
    .await
    .expect("join");

    leave_session(
        &participants,
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
        &sessions,
        &rooms,
        JoinSession {
            actor: owner,
            session_id: session.id,
        },
    )
    .await
    .expect("rejoin");
    assert_ne!(rejoined.id, joined.id, "a rejoin is a new participation");
    assert!(rejoined.is_active());
}

#[tokio::test]
async fn leaving_another_actors_participant_is_not_found() {
    let (rooms, sessions, participants, _room, session, owner) = live_fixture();
    let joined = join_session(
        &participants,
        &sessions,
        &rooms,
        JoinSession {
            actor: owner,
            session_id: session.id,
        },
    )
    .await
    .expect("join");

    let denied = leave_session(
        &participants,
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
            actor: member,
            room_id: room.id,
            scheduled_start_at: None,
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
            actor: stranger,
            room_id: room.id,
            scheduled_start_at: None,
        },
    )
    .await;
    assert!(
        matches!(hidden, Err(super::SessionOperationError::NotFound)),
        "a non-member must not learn the room exists, got {hidden:?}"
    );
}

#[tokio::test]
async fn invalid_transition_is_rejected_without_state_change() {
    let rooms = FakeRoomStore::default();
    let sessions = FakeSessionStore::default();
    let owner = actor();
    let room = rooms.seed_room(owner);

    let session = scheduled(room.id);
    sessions.seed(session.clone());

    // Ending a scheduled session is invalid.
    let ended = end_session(
        &sessions,
        &rooms,
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
    let owner = actor();
    let room = rooms.seed_room(owner);

    let scheduled_session = schedule_session(
        &sessions,
        &rooms,
        ScheduleSession {
            actor: owner,
            room_id: room.id,
            scheduled_start_at: None,
        },
    )
    .await
    .expect("schedule");
    assert_eq!(scheduled_session.status, SessionStatus::Scheduled);

    let live = start_session(
        &sessions,
        &rooms,
        StartSession {
            actor: owner,
            session_id: scheduled_session.id,
        },
    )
    .await
    .expect("start");
    assert_eq!(live.status, SessionStatus::Live);

    let ended = end_session(
        &sessions,
        &rooms,
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
            actor: owner,
            session_id: first.id,
        },
    )
    .await
    .expect("first start wins");

    let losing = start_session(
        &sessions,
        &rooms,
        StartSession {
            actor: owner,
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
