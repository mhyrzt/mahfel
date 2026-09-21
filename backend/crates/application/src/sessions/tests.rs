use super::{
    CancelSession, EndSession, GetSession, ListSessions, ScheduleSession, StartSession,
    cancel_session, end_session, get_session, list_sessions, schedule_session, start_session,
};
use crate::ports::{RoomStore, SessionStore, StoreError};
use mahfel_domain::{
    ActorId, Room, RoomId, RoomMember, RoomRole, RoomTitle, RoomVisibility, Session, SessionError,
    SessionId, SessionStatus,
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
        let now = OffsetDateTime::now_utc();
        let room = Room::create(
            RoomTitle::new("Algebra".to_owned()).expect("valid title"),
            owner,
            RoomVisibility::InviteOnly,
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
    async fn get(&self, id: RoomId) -> Result<Option<Room>, StoreError> {
        Ok(self
            .rooms
            .lock()
            .expect("lock")
            .iter()
            .find(|room| room.id == id)
            .cloned())
    }

    async fn insert(&self, room: &Room) -> Result<(), StoreError> {
        self.rooms.lock().expect("lock").push(room.clone());
        Ok(())
    }

    async fn save(&self, room: &Room) -> Result<(), StoreError> {
        let mut rooms = self.rooms.lock().expect("lock");
        let slot = rooms
            .iter_mut()
            .find(|existing| existing.id == room.id)
            .ok_or_else(|| StoreError::NotFound(room.id.to_string()))?;
        *slot = room.clone();
        Ok(())
    }

    async fn create_with_manager(
        &self,
        room: &Room,
        manager: &RoomMember,
    ) -> Result<(), StoreError> {
        self.rooms.lock().expect("lock").push(room.clone());
        self.members.lock().expect("lock").push(manager.clone());
        Ok(())
    }

    async fn list_for_member(
        &self,
        actor_id: ActorId,
        _limit: u64,
        _offset: u64,
    ) -> Result<Vec<Room>, StoreError> {
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
    }

    async fn get_member(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> Result<Option<RoomMember>, StoreError> {
        Ok(self
            .members
            .lock()
            .expect("lock")
            .iter()
            .find(|member| member.room_id == room_id && member.actor_id == actor_id)
            .cloned())
    }

    async fn list_members(&self, room_id: RoomId) -> Result<Vec<RoomMember>, StoreError> {
        Ok(self
            .members
            .lock()
            .expect("lock")
            .iter()
            .filter(|member| member.room_id == room_id)
            .cloned()
            .collect())
    }

    async fn add_member(&self, member: &RoomMember) -> Result<(), StoreError> {
        self.members.lock().expect("lock").push(member.clone());
        Ok(())
    }

    async fn remove_member(&self, room_id: RoomId, actor_id: ActorId) -> Result<bool, StoreError> {
        let mut members = self.members.lock().expect("lock");
        let before = members.len();
        members.retain(|member| !(member.room_id == room_id && member.actor_id == actor_id));
        Ok(members.len() < before)
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
    async fn get(&self, id: SessionId) -> Result<Option<Session>, StoreError> {
        Ok(self
            .sessions
            .lock()
            .expect("lock")
            .iter()
            .find(|session| session.id == id)
            .cloned())
    }

    async fn insert(&self, session: &Session) -> Result<(), StoreError> {
        if session.status == SessionStatus::Live && self.has_live(session.room_id, None) {
            return Err(StoreError::Conflict("live session exists".to_owned()));
        }
        self.sessions.lock().expect("lock").push(session.clone());
        Ok(())
    }

    async fn save(&self, session: &Session) -> Result<(), StoreError> {
        if session.status == SessionStatus::Live && self.has_live(session.room_id, Some(session.id))
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
    }

    async fn list_for_room(&self, room_id: RoomId) -> Result<Vec<Session>, StoreError> {
        Ok(self
            .sessions
            .lock()
            .expect("lock")
            .iter()
            .filter(|session| session.room_id == room_id)
            .cloned()
            .collect())
    }
}

fn actor() -> ActorId {
    ActorId::new()
}

fn scheduled(room_id: RoomId) -> Session {
    Session::schedule(room_id, None, OffsetDateTime::now_utc())
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
        matches!(denied, Err(super::ScheduleSessionError::Forbidden)),
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
        matches!(hidden, Err(super::ScheduleSessionError::NotFound)),
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
            Err(super::EndSessionError::Domain(SessionError::CannotEnd(_)))
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
            Err(super::CancelSessionError::Domain(
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
        matches!(losing, Err(super::StartSessionError::Conflict)),
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
