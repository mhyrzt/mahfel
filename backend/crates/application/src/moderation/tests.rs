//!
//! These exercise the ordering the module promises: capability first, then the
//! authoritative target lookup, then durable state, then media enforcement.

use super::error::ModerationError;
use super::mute::{MuteCommand, MutedSource, mute, unmute};
use super::remove::{BlockCommand, RemoveCommand, block, remove_participant, unblock};
use crate::ports::{
    BlockStore, BoxFuture, Capability, CapabilityContext, CapabilityResolver, CapabilitySet,
    MediaGateway, MediaGatewayError, Membership, MembershipStore, ParticipantStore, RoomStore,
    SessionStore, StoreError,
};
use mahfel_domain::{
    ActorId, Block, BlockId, BlockScope, Participant, ParticipantId, ProducerSource, Room, RoomId,
    RoomMember, RoomRole, RoomTitle, RoomVisibility, Session, SessionId,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use time::OffsetDateTime;

#[tokio::test]
async fn moderator_cannot_target_themself() {
    let f = fixture();
    f.participants.seed_active(f.session.id, f.manager);
    let now = OffsetDateTime::now_utc();
    let removed = remove_participant(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        RemoveCommand {
            actor: f.manager,
            session_id: f.session.id,
            subject: f.manager,
        },
        now,
    )
    .await;
    assert!(matches!(removed, Err(ModerationError::Forbidden)));
    let muted = mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        mute_command(f.manager, f.manager, f.session.clone()),
        now,
    )
    .await;
    assert!(matches!(muted, Err(ModerationError::Forbidden)));
    let blocked = block(
        &f.rooms,
        &f.sessions,
        &f.participants,
        &f.blocks,
        &f.memberships,
        &f.media,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Session(f.session.id),
            subject: f.manager,
            reason: None,
            expires_at: None,
        },
        now,
    )
    .await;
    assert!(matches!(blocked, Err(ModerationError::Forbidden)));
    let unblocked = unblock(
        &f.rooms,
        &f.sessions,
        &f.blocks,
        &f.memberships,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Session(f.session.id),
            subject: f.manager,
            reason: None,
            expires_at: None,
        },
        now,
    )
    .await;
    assert!(matches!(unblocked, Err(ModerationError::Forbidden)));
    assert!(
        f.participants
            .get_active(f.session.id, f.manager)
            .await
            .expect("lookup")
            .is_some()
    );
}

#[derive(Clone, Default)]
struct FakeRooms {
    rooms: Arc<Mutex<Vec<Room>>>,
    members: Arc<Mutex<Vec<RoomMember>>>,
}

impl FakeRooms {
    fn seed(&self, owner: ActorId) -> Room {
        let now = OffsetDateTime::now_utc();
        let room = Room::create(
            RoomTitle::new("Analytical Engines".to_owned()).expect("title"),
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

impl RoomStore for FakeRooms {
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
            if let Some(slot) = rooms.iter_mut().find(|existing| existing.id == room.id) {
                *slot = room.clone();
            }
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
        _actor_id: ActorId,
        _limit: u64,
        _offset: u64,
    ) -> BoxFuture<'_, Result<Vec<Room>, StoreError>> {
        Box::pin(async move { Ok(Vec::new()) })
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
struct FakeSessions {
    sessions: Arc<Mutex<Vec<Session>>>,
}

impl FakeSessions {
    fn seed_live(&self, room_id: RoomId) -> Session {
        let now = OffsetDateTime::now_utc();
        let mut session = Session::schedule(room_id, None, now);
        session.start(now).expect("start");
        self.sessions.lock().expect("lock").push(session.clone());
        session
    }
}

impl SessionStore for FakeSessions {
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
            self.sessions.lock().expect("lock").push(session.clone());
            Ok(())
        })
    }

    fn save<'a>(&'a self, session: &'a Session) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut sessions = self.sessions.lock().expect("lock");
            if let Some(slot) = sessions
                .iter_mut()
                .find(|existing| existing.id == session.id)
            {
                *slot = session.clone();
            }
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

#[derive(Clone, Default)]
struct FakeParticipants {
    participants: Arc<Mutex<Vec<Participant>>>,
}

impl FakeParticipants {
    fn seed_active(&self, session_id: SessionId, actor: ActorId) -> Participant {
        let participant = Participant::join(session_id, actor, OffsetDateTime::now_utc());
        self.participants
            .lock()
            .expect("lock")
            .push(participant.clone());
        participant
    }

    fn current(&self, id: ParticipantId) -> Participant {
        self.participants
            .lock()
            .expect("lock")
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .expect("participant")
    }
}

impl ParticipantStore for FakeParticipants {
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
        Box::pin(async move {
            Ok(self
                .participants
                .lock()
                .expect("lock")
                .iter()
                .find(|p| p.session_id == session_id && p.actor_id == actor_id && p.is_active())
                .cloned())
        })
    }

    fn insert<'a>(&'a self, participant: &'a Participant) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
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

#[derive(Clone, Default)]
struct FakeBlocks {
    blocks: Arc<Mutex<Vec<Block>>>,
}

impl BlockStore for FakeBlocks {
    fn get_active(
        &self,
        scope: BlockScope,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<Block>, StoreError>> {
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

    fn insert<'a>(&'a self, block: &'a Block) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            self.blocks.lock().expect("lock").push(block.clone());
            Ok(())
        })
    }

    fn extend(
        &self,
        id: BlockId,
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
        id: BlockId,
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
        room_id: RoomId,
        _limit: u32,
    ) -> BoxFuture<'_, Result<Vec<Block>, StoreError>> {
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
    ) -> BoxFuture<'_, Result<Vec<Block>, StoreError>> {
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

#[derive(Clone, Default)]
struct FakeMemberships {
    roles: Arc<Mutex<HashMap<(RoomId, ActorId), RoomRole>>>,
}

impl FakeMemberships {
    fn grant(&self, room_id: RoomId, actor: ActorId, role: RoomRole) {
        self.roles
            .lock()
            .expect("lock")
            .insert((room_id, actor), role);
    }
}

impl MembershipStore for FakeMemberships {
    fn role_of(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Membership, StoreError>> {
        Box::pin(async move {
            Ok(self
                .roles
                .lock()
                .expect("lock")
                .get(&(room_id, actor_id))
                .copied())
        })
    }
}

/// Records enforcement calls, so a test can prove what reached the media plane.
#[derive(Clone, Default)]
struct FakeMedia {
    paused: Arc<Mutex<Vec<(RoomId, ActorId, ProducerSource)>>>,
    resumed: Arc<Mutex<Vec<(RoomId, ActorId, ProducerSource)>>>,
    closed: Arc<Mutex<Vec<(RoomId, ActorId)>>>,
    /// Source-targeted closes, which the reconcile uses.
    closed_sources: Arc<Mutex<Vec<(RoomId, ActorId, ProducerSource)>>>,
    /// Producers reported as affected, so a test can distinguish "enforced
    /// nothing" from "not asked".
    affected: usize,
    fail: bool,
}

impl MediaGateway for FakeMedia {
    fn create_room(&self, _room: RoomId) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move { Ok(()) })
    }

    fn close_room(&self, _room: RoomId) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move { Ok(()) })
    }

    fn pause_producer(
        &self,
        _room: RoomId,
        _producer_id: &str,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move { Ok(()) })
    }

    fn resume_producer(
        &self,
        _room: RoomId,
        _producer_id: &str,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move { Ok(()) })
    }

    fn close_participant(
        &self,
        room: RoomId,
        participant_id: ActorId,
    ) -> BoxFuture<'_, Result<(), MediaGatewayError>> {
        Box::pin(async move {
            if self.fail {
                return Err(MediaGatewayError::transport("media plane down"));
            }
            self.closed
                .lock()
                .expect("lock")
                .push((room, participant_id));
            Ok(())
        })
    }

    fn close_participant_source(
        &self,
        room: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>> {
        Box::pin(async move {
            if self.fail {
                return Err(MediaGatewayError::transport("media plane down"));
            }
            self.closed_sources
                .lock()
                .expect("lock")
                .push((room, participant_id, source));
            Ok(self.affected)
        })
    }

    fn pause_participant_source(
        &self,
        room: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>> {
        Box::pin(async move {
            if self.fail {
                return Err(MediaGatewayError::transport("media plane down"));
            }
            self.paused
                .lock()
                .expect("lock")
                .push((room, participant_id, source));
            Ok(self.affected)
        })
    }

    fn resume_participant_source(
        &self,
        room: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> BoxFuture<'_, Result<usize, MediaGatewayError>> {
        Box::pin(async move {
            if self.fail {
                return Err(MediaGatewayError::transport("media plane down"));
            }
            self.resumed
                .lock()
                .expect("lock")
                .push((room, participant_id, source));
            Ok(self.affected)
        })
    }
}

struct Fixture {
    rooms: FakeRooms,
    sessions: FakeSessions,
    participants: FakeParticipants,
    blocks: FakeBlocks,
    memberships: FakeMemberships,
    media: FakeMedia,
    manager: ActorId,
    member: ActorId,
    outsider: ActorId,
    room: Room,
    session: Session,
}

fn fixture() -> Fixture {
    let rooms = FakeRooms::default();
    let sessions = FakeSessions::default();
    let memberships = FakeMemberships::default();
    let manager = ActorId::new();
    let room = rooms.seed(manager);
    memberships.grant(room.id, manager, RoomRole::Manager);

    let member = ActorId::new();
    memberships.grant(room.id, member, RoomRole::Member);
    rooms.add_member(room.id, member, RoomRole::Member);

    let session = sessions.seed_live(room.id);
    Fixture {
        rooms,
        sessions,
        participants: FakeParticipants::default(),
        blocks: FakeBlocks::default(),
        memberships,
        media: FakeMedia::default(),
        manager,
        member,
        outsider: ActorId::new(),
        room,
        session,
    }
}

fn mute_command(actor: ActorId, subject: ActorId, session: Session) -> MuteCommand {
    MuteCommand {
        actor,
        session_id: session.id,
        subject,
        kind: mahfel_domain::MuteKind::Temporary,
        source: MutedSource::Microphone,
    }
}

#[tokio::test]
async fn a_member_cannot_mute_another_participant() {
    let f = fixture();
    let victim = ActorId::new();
    f.participants.seed_active(f.session.id, victim);

    let mut command = mute_command(f.manager, victim, f.session.clone());
    command.actor = f.member;

    let result = mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        command,
        OffsetDateTime::now_utc(),
    )
    .await;

    assert!(
        matches!(result, Err(ModerationError::Forbidden)),
        "a plain member must not moderate"
    );
    assert!(
        f.media.paused.lock().expect("lock").is_empty(),
        "a denied action must not reach the media plane"
    );
}

#[tokio::test]
async fn an_outsider_cannot_mute() {
    let f = fixture();
    let victim = ActorId::new();
    f.participants.seed_active(f.session.id, victim);

    let mut command = mute_command(f.manager, victim, f.session.clone());
    command.actor = f.outsider;

    let result = mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        command,
        OffsetDateTime::now_utc(),
    )
    .await;
    assert!(matches!(result, Err(ModerationError::Forbidden)));
}

#[tokio::test]
async fn a_manager_can_mute() {
    let f = fixture();
    let victim = f.participants.seed_active(f.session.id, f.member);

    let outcome = mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        mute_command(f.manager, f.member, f.session.clone()),
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("manager mute");

    assert_eq!(outcome.kind, mahfel_domain::MuteKind::Temporary);
    assert_eq!(
        f.media.paused.lock().expect("lock").as_slice(),
        &[(f.room.id, f.member, ProducerSource::Microphone)],
        "muting a microphone must pause that source only"
    );
    assert!(
        f.participants.current(victim.id).is_muted(),
        "the mute is durable, not only enforced"
    );
}

#[tokio::test]
async fn muting_a_non_participant_fails_before_any_media_call() {
    let f = fixture();

    let result = mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        mute_command(f.manager, f.member, f.session.clone()),
        OffsetDateTime::now_utc(),
    )
    .await;

    assert!(
        matches!(result, Err(ModerationError::NotFound)),
        "an unknown participant is not found"
    );
    assert!(
        f.media.paused.lock().expect("lock").is_empty(),
        "identity is resolved before enforcement, so nothing reached the media plane"
    );
}

#[tokio::test]
async fn muting_a_participant_who_left_fails() {
    let f = fixture();
    let mut participant = f.participants.seed_active(f.session.id, f.member);
    participant.leave(OffsetDateTime::now_utc()).expect("leave");
    f.participants.save(&participant).await.expect("save left");

    let result = mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        mute_command(f.manager, f.member, f.session.clone()),
        OffsetDateTime::now_utc(),
    )
    .await;
    assert!(matches!(result, Err(ModerationError::NotFound)));
}

#[tokio::test]
async fn the_target_clears_their_own_temporary_mute() {
    let f = fixture();
    f.participants.seed_active(f.session.id, f.member);

    mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        mute_command(f.manager, f.member, f.session.clone()),
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("mute");

    let outcome = unmute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        mute_command(f.member, f.member, f.session.clone()),
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("self unmute");

    assert_eq!(outcome.producers_affected, 0);
    assert_eq!(
        f.media.resumed.lock().expect("lock").as_slice(),
        &[(f.room.id, f.member, ProducerSource::Microphone)],
        "unmuting resumes the same source it paused"
    );
}

#[tokio::test]
async fn the_target_cannot_clear_a_permanent_mute() {
    let f = fixture();
    f.participants.seed_active(f.session.id, f.member);

    let mut command = mute_command(f.manager, f.member, f.session.clone());
    command.kind = mahfel_domain::MuteKind::Permanent;
    mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        command,
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("permanent mute");

    let result = unmute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        mute_command(f.member, f.member, f.session.clone()),
        OffsetDateTime::now_utc(),
    )
    .await;

    assert!(
        matches!(result, Err(ModerationError::PermanentMuteNeedsModerator)),
        "a permanent mute is not the target's to clear, got {result:?}"
    );
    assert!(
        f.media.resumed.lock().expect("lock").is_empty(),
        "a refused unmute must not resume anything"
    );
}

#[tokio::test]
async fn a_manager_clears_a_permanent_mute() {
    let f = fixture();
    f.participants.seed_active(f.session.id, f.member);

    let mut command = mute_command(f.manager, f.member, f.session.clone());
    command.kind = mahfel_domain::MuteKind::Permanent;
    mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        command,
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("permanent mute");

    unmute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        mute_command(f.manager, f.member, f.session.clone()),
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("manager unmute");
}

#[tokio::test]
async fn stopping_a_screen_share_leaves_the_microphone_alone() {
    let f = fixture();
    f.participants.seed_active(f.session.id, f.member);

    let mut command = mute_command(f.manager, f.member, f.session.clone());
    command.source = MutedSource::Screen;
    mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        command,
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("stop screen share");

    assert_eq!(
        f.media.paused.lock().expect("lock").as_slice(),
        &[(f.room.id, f.member, ProducerSource::Screen)],
        "only the screen source is affected"
    );
}

#[tokio::test]
async fn a_media_failure_is_reported_as_degraded_and_leaves_the_mute_recorded() {
    // The mute is recorded and the caller is told enforcement did not happen, so
    // the target can still be told to stop even when the media plane is down.
    // Failing the call outright would leave the participant muted while their
    // client kept publishing and nobody was informed.
    let f = fixture();
    let victim = f.participants.seed_active(f.session.id, f.member);
    let failing = FakeMedia {
        fail: true,
        ..FakeMedia::default()
    };

    let outcome = mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &failing,
        mute_command(f.manager, f.member, f.session.clone()),
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("a media failure must not fail the mute itself");

    assert!(
        outcome.degraded,
        "the caller must learn that enforcement did not happen"
    );
    assert_eq!(outcome.producers_affected, 0);
    assert!(
        f.participants.current(victim.id).is_muted(),
        "the durable mute stands, so an operator can reconcile"
    );
}

#[tokio::test]
async fn a_healthy_mute_is_not_degraded() {
    let f = fixture();
    f.participants.seed_active(f.session.id, f.member);

    let outcome = mute(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        mute_command(f.manager, f.member, f.session.clone()),
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("mute");

    assert!(!outcome.degraded);
}

#[tokio::test]
async fn removing_ends_participation_and_closes_producers() {
    let f = fixture();
    let victim = f.participants.seed_active(f.session.id, f.member);

    remove_participant(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        RemoveCommand {
            actor: f.manager,
            session_id: f.session.id,
            subject: f.member,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("remove");

    assert!(!f.participants.current(victim.id).is_active());
    assert_eq!(
        f.media.closed.lock().expect("lock").as_slice(),
        &[(f.room.id, f.member)],
        "removal closes the principal's producers once"
    );
}

#[tokio::test]
async fn removing_does_not_block_a_rejoin() {
    // Remove is orthogonal to block: the principal may come back.
    let f = fixture();
    f.participants.seed_active(f.session.id, f.member);

    remove_participant(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        RemoveCommand {
            actor: f.manager,
            session_id: f.session.id,
            subject: f.member,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("remove");

    let active = f
        .blocks
        .get_active(BlockScope::Room(f.room.id), f.member)
        .await
        .expect("lookup");
    assert!(active.is_none(), "removal must not silently create a block");
}

#[tokio::test]
async fn a_member_cannot_remove() {
    let f = fixture();
    f.participants.seed_active(f.session.id, f.manager);

    let result = remove_participant(
        &f.participants,
        &f.sessions,
        &f.memberships,
        &f.media,
        RemoveCommand {
            actor: f.member,
            session_id: f.session.id,
            subject: f.manager,
        },
        OffsetDateTime::now_utc(),
    )
    .await;

    assert!(matches!(result, Err(ModerationError::Forbidden)));
    assert!(
        f.media.closed.lock().expect("lock").is_empty(),
        "a denied removal must not close anything"
    );
}

#[tokio::test]
async fn blocking_creates_one_durable_block() {
    let f = fixture();

    let outcome = block(
        &f.rooms,
        &f.sessions,
        &f.participants,
        &f.blocks,
        &f.memberships,
        &f.media,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Room(f.room.id),
            subject: f.outsider,
            reason: Some("spam".to_owned()),
            expires_at: None,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("block");

    assert!(outcome.created, "a first block is created");
    let active = f
        .blocks
        .get_active(BlockScope::Room(f.room.id), f.outsider)
        .await
        .expect("lookup");
    assert!(active.is_some(), "a room block is durable");
}

#[tokio::test]
async fn reblocking_extends_rather_than_stacking() {
    let f = fixture();
    let now = OffsetDateTime::now_utc();

    block(
        &f.rooms,
        &f.sessions,
        &f.participants,
        &f.blocks,
        &f.memberships,
        &f.media,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Room(f.room.id),
            subject: f.outsider,
            reason: None,
            expires_at: None,
        },
        now,
    )
    .await
    .expect("first block");

    let outcome = block(
        &f.rooms,
        &f.sessions,
        &f.participants,
        &f.blocks,
        &f.memberships,
        &f.media,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Room(f.room.id),
            subject: f.outsider,
            reason: None,
            expires_at: Some(now + time::Duration::hours(1)),
        },
        now,
    )
    .await
    .expect("second block");

    assert!(!outcome.created, "a re-block extends instead of creating");
    assert_eq!(
        f.blocks.blocks.lock().expect("lock").len(),
        1,
        "a re-block must not stack a second row"
    );
}

#[tokio::test]
async fn a_room_block_and_a_session_block_are_independent() {
    // The lift path is why this matters: lifting the room block must not clear
    // an unrelated session block.
    let f = fixture();
    let now = OffsetDateTime::now_utc();

    block(
        &f.rooms,
        &f.sessions,
        &f.participants,
        &f.blocks,
        &f.memberships,
        &f.media,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Session(f.session.id),
            subject: f.outsider,
            reason: None,
            expires_at: None,
        },
        now,
    )
    .await
    .expect("session block");

    block(
        &f.rooms,
        &f.sessions,
        &f.participants,
        &f.blocks,
        &f.memberships,
        &f.media,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Room(f.room.id),
            subject: f.outsider,
            reason: None,
            expires_at: None,
        },
        now,
    )
    .await
    .expect("room block");

    unblock(
        &f.rooms,
        &f.sessions,
        &f.blocks,
        &f.memberships,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Room(f.room.id),
            subject: f.outsider,
            reason: None,
            expires_at: None,
        },
        now,
    )
    .await
    .expect("lift room block");

    let session_block = f
        .blocks
        .get_active(BlockScope::Session(f.session.id), f.outsider)
        .await
        .expect("lookup");
    assert!(
        session_block.is_some(),
        "lifting the room block must not disturb the session block"
    );
}

#[tokio::test]
async fn a_member_cannot_block() {
    let f = fixture();

    let result = block(
        &f.rooms,
        &f.sessions,
        &f.participants,
        &f.blocks,
        &f.memberships,
        &f.media,
        BlockCommand {
            actor: f.member,
            scope: BlockScope::Room(f.room.id),
            subject: f.outsider,
            reason: None,
            expires_at: None,
        },
        OffsetDateTime::now_utc(),
    )
    .await;
    assert!(matches!(result, Err(ModerationError::Forbidden)));
}

#[tokio::test]
async fn blocking_from_an_unknown_session_fails() {
    let f = fixture();

    let result = block(
        &f.rooms,
        &f.sessions,
        &f.participants,
        &f.blocks,
        &f.memberships,
        &f.media,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Session(SessionId::new()),
            subject: f.outsider,
            reason: None,
            expires_at: None,
        },
        OffsetDateTime::now_utc(),
    )
    .await;
    assert!(matches!(result, Err(ModerationError::NotFound)));
}

/// A resolver that grants nothing, to prove the capability check is real.
struct DenyAllResolver;

impl CapabilityResolver for DenyAllResolver {
    fn resolve(
        &self,
        _principal: &crate::ports::Principal,
        _context: CapabilityContext,
    ) -> CapabilitySet {
        CapabilitySet::default()
    }
}

#[test]
fn the_policy_grants_moderation_to_a_manager_but_not_a_member() {
    // Guards the assumption the use cases rest on: a manager's membership is
    // what grants MuteOthers, so a member's denial is not incidental.
    let resolver = crate::policy::ServerCapabilityResolver;
    let principal = crate::ports::Principal {
        actor_id: ActorId::new(),
        kind: mahfel_domain::ActorKind::User,
        display_name: None,
        avatar_url: None,
        email: None,
        email_verified: false,
        username: None,
        roles: Vec::new(),
    };

    let manager = resolver.resolve(
        &principal,
        CapabilityContext::in_room(Some(RoomRole::Manager)),
    );
    assert!(manager.has(Capability::MuteOthers));
    assert!(manager.has(Capability::RemoveParticipant));

    let member = resolver.resolve(
        &principal,
        CapabilityContext::in_room(Some(RoomRole::Member)),
    );
    assert!(!member.has(Capability::MuteOthers));
    assert!(!member.has(Capability::RemoveParticipant));

    let deny = DenyAllResolver.resolve(&principal, CapabilityContext::global());
    assert!(!deny.has(Capability::MuteOthers));
}

#[tokio::test]
async fn a_withdrawn_capability_closes_every_source() {
    // Membership removed: every media capability goes, so every source closes.
    let f = fixture();
    f.participants.seed_active(f.session.id, f.member);
    // Both stores, because production resolves membership through the room
    // store: the two are one fact there and a fake must not let them disagree.
    f.memberships
        .roles
        .lock()
        .expect("lock")
        .remove(&(f.room.id, f.member));
    f.rooms
        .remove_member(f.room.id, f.member)
        .await
        .expect("remove membership");

    let media = FakeMedia {
        // One producer per source, so a close has something to report. A fake
        // reporting zero would prove only that zero is zero.
        affected: 1,
        ..FakeMedia::default()
    };
    let outcome = super::reconcile::reconcile_media(&f.rooms, &media, f.member, f.room.id)
        .await
        .expect("reconcile");

    assert!(outcome.closed_microphone);
    assert!(outcome.closed_camera);
    assert!(outcome.closed_screen);
    let sources: Vec<ProducerSource> = media
        .closed_sources
        .lock()
        .expect("lock")
        .iter()
        .map(|(_, _, source)| *source)
        .collect();
    assert_eq!(sources.len(), 3, "every source is reconciled");
}

#[tokio::test]
async fn a_demotion_closes_nothing() {
    // A member keeps PublishMedia and ShareScreen, so there is nothing to close
    // and no media call to make. This is why the reconcile is not a sweep.
    let f = fixture();
    f.participants.seed_active(f.session.id, f.member);
    f.memberships.grant(f.room.id, f.member, RoomRole::Member);

    let outcome = super::reconcile::reconcile_media(&f.rooms, &f.media, f.member, f.room.id)
        .await
        .expect("reconcile");

    assert!(!outcome.closed_anything());
    assert!(
        f.media.closed_sources.lock().expect("lock").is_empty(),
        "a demotion must not touch the media plane"
    );
}

#[tokio::test]
async fn a_principal_who_holds_the_capability_is_left_alone() {
    let f = fixture();
    f.participants.seed_active(f.session.id, f.member);

    let outcome = super::reconcile::reconcile_media(&f.rooms, &f.media, f.member, f.room.id)
        .await
        .expect("reconcile");

    assert!(!outcome.closed_anything());
    assert!(f.media.closed_sources.lock().expect("lock").is_empty());
}

#[tokio::test]
async fn the_capability_for_a_source_is_the_one_that_permits_it() {
    // Screen is a separate capability from speaking, which is what makes the
    // source field load-bearing at the produce gate too.
    use crate::ports::Capability;
    assert_eq!(
        super::reconcile::capability_for(ProducerSource::Microphone),
        Capability::PublishMedia
    );
    assert_eq!(
        super::reconcile::capability_for(ProducerSource::Camera),
        Capability::PublishMedia
    );
    assert_eq!(
        super::reconcile::capability_for(ProducerSource::Screen),
        Capability::ShareScreen
    );
}

#[tokio::test]
async fn blocking_ejects_a_principal_who_is_present() {
    // A block bars rejoining, so one that left the subject publishing would not
    // be a block.
    let f = fixture();
    let victim = f.participants.seed_active(f.session.id, f.member);

    let outcome = block(
        &f.rooms,
        &f.sessions,
        &f.participants,
        &f.blocks,
        &f.memberships,
        &f.media,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Room(f.room.id),
            subject: f.member,
            reason: None,
            expires_at: None,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("block");

    assert!(outcome.created);
    assert_eq!(outcome.ejected_from, 1, "the subject was ejected");
    assert!(
        !f.participants.current(victim.id).is_active(),
        "a blocked principal's participation ends"
    );
    assert_eq!(
        f.media.closed.lock().expect("lock").as_slice(),
        &[(f.room.id, f.member)],
        "their producers close"
    );
}

#[tokio::test]
async fn blocking_an_absent_principal_ejects_nothing() {
    let f = fixture();

    let outcome = block(
        &f.rooms,
        &f.sessions,
        &f.participants,
        &f.blocks,
        &f.memberships,
        &f.media,
        BlockCommand {
            actor: f.manager,
            scope: BlockScope::Room(f.room.id),
            subject: f.outsider,
            reason: None,
            expires_at: None,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("block");

    assert_eq!(outcome.ejected_from, 0);
    assert!(f.media.closed.lock().expect("lock").is_empty());
}
