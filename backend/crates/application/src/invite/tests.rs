//! Redemption tests: the AC 1-5 surface.
//!
//! These live in the application crate because redemption is the use case that
//! has to hold the non-enumerating and rate-limit properties together.

use super::rate_limit::{AttemptLimiter, MAX_ATTEMPTS_PER_PRINCIPAL};
use super::redeem::{RedeemError, RedeemInviteLink, Redeemed, redeem_invite_link};
use crate::admission::AdmissionDecision;
use crate::ports::{
    AdmissionStore, BoxFuture, CoordinationError, CoordinationStore, InviteStore, RoomStore,
    SessionStore, StoreError,
};
use mahfel_domain::{
    ActorId, AdmissionOrigin, AdmissionPolicy, AdmissionRequest, AdmissionRequestId,
    AdmissionRequestStatus, AdmissionTarget, InviteCode, InviteLink, InviteLinkId, InviteTarget,
    InviteTargetKind, Room, RoomId, RoomMember, RoomRole, RoomStatus, RoomTitle, RoomVisibility,
    Session, SessionId,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use time::OffsetDateTime;

#[derive(Clone, Default)]
struct FakeRooms {
    rooms: Arc<Mutex<Vec<Room>>>,
    members: Arc<Mutex<Vec<RoomMember>>>,
}

impl FakeRooms {
    fn seed_room(&self, owner: ActorId, visibility: RoomVisibility) -> Room {
        let now = OffsetDateTime::now_utc();
        let room = Room::create(
            RoomTitle::new("Analytical Engines".to_owned()).expect("title"),
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

    fn is_member(&self, room_id: RoomId, actor: ActorId) -> bool {
        self.members
            .lock()
            .expect("lock")
            .iter()
            .any(|member| member.room_id == room_id && member.actor_id == actor)
    }

    fn archive(&self, room_id: RoomId) {
        let mut rooms = self.rooms.lock().expect("lock");
        if let Some(room) = rooms.iter_mut().find(|room| room.id == room_id) {
            room.status = RoomStatus::Archived;
        }
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
            let mut members = self.members.lock().expect("lock");
            if members.iter().any(|existing| {
                existing.room_id == member.room_id && existing.actor_id == member.actor_id
            }) {
                return Err(StoreError::Conflict("already a member".to_owned()));
            }
            members.push(member.clone());
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
    fn seed(&self, session: Session) {
        self.sessions.lock().expect("lock").push(session);
    }

    fn end(&self, session_id: SessionId) {
        let mut sessions = self.sessions.lock().expect("lock");
        if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id) {
            session.end(OffsetDateTime::now_utc()).expect("end");
        }
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
struct FakeInvites {
    links: Arc<Mutex<Vec<InviteLink>>>,
}

impl FakeInvites {
    fn seed(&self, link: InviteLink) {
        self.links.lock().expect("lock").push(link);
    }
}

impl InviteStore for FakeInvites {
    fn insert<'a>(&'a self, link: &'a InviteLink) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            self.links.lock().expect("lock").push(link.clone());
            Ok(())
        })
    }

    fn get_by_code(
        &self,
        code: InviteCode,
    ) -> BoxFuture<'_, Result<Option<InviteLink>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .links
                .lock()
                .expect("lock")
                .iter()
                .find(|link| link.code == code)
                .cloned())
        })
    }

    fn get(&self, id: InviteLinkId) -> BoxFuture<'_, Result<Option<InviteLink>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .links
                .lock()
                .expect("lock")
                .iter()
                .find(|link| link.id == id)
                .cloned())
        })
    }

    fn list_for_target(
        &self,
        _target_kind: InviteTargetKind,
        _target_id: uuid::Uuid,
        _limit: u32,
    ) -> BoxFuture<'_, Result<Vec<InviteLink>, StoreError>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn revoke_if_active(
        &self,
        _id: InviteLinkId,
        _revoked_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move { Ok(false) })
    }

    fn rotate<'a>(
        &'a self,
        _previous: InviteLinkId,
        _revoked_at: OffsetDateTime,
        _replacement: &'a InviteLink,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move { Ok(()) })
    }
}

#[derive(Clone, Default)]
struct FakeAdmissions {
    requests: Arc<Mutex<Vec<AdmissionRequest>>>,
}

impl FakeAdmissions {
    fn all(&self) -> Vec<AdmissionRequest> {
        self.requests.lock().expect("lock").clone()
    }
}

impl AdmissionStore for FakeAdmissions {
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
        Box::pin(async move {
            Ok(self
                .requests
                .lock()
                .expect("lock")
                .iter()
                .find(|request| {
                    request.target == target && request.actor_id == actor_id && request.is_pending()
                })
                .cloned())
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
        _limit: u32,
    ) -> BoxFuture<'_, Result<Vec<AdmissionRequest>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .requests
                .lock()
                .expect("lock")
                .iter()
                .filter(|request| {
                    request.is_pending()
                        && matches!(request.target, AdmissionTarget::Session(id) if id == session_id)
                })
                .cloned()
                .collect())
        })
    }

    fn list_pending_for_room(
        &self,
        room_id: RoomId,
        _limit: u32,
    ) -> BoxFuture<'_, Result<Vec<AdmissionRequest>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .requests
                .lock()
                .expect("lock")
                .iter()
                .filter(|request| {
                    request.is_pending()
                        && matches!(request.target, AdmissionTarget::Room(id) if id == room_id)
                })
                .cloned()
                .collect())
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
struct FakeCoordination {
    counts: Arc<Mutex<Vec<(String, u64)>>>,
}

impl CoordinationStore for FakeCoordination {
    fn set_with_ttl<'a>(
        &'a self,
        _key: &'a str,
        _value: &'a [u8],
        _ttl: Duration,
    ) -> BoxFuture<'a, Result<(), CoordinationError>> {
        Box::pin(async move { Ok(()) })
    }

    fn get<'a>(
        &'a self,
        _key: &'a str,
    ) -> BoxFuture<'a, Result<Option<Vec<u8>>, CoordinationError>> {
        Box::pin(async move { Ok(None) })
    }

    fn delete<'a>(&'a self, _key: &'a str) -> BoxFuture<'a, Result<(), CoordinationError>> {
        Box::pin(async move { Ok(()) })
    }

    fn publish<'a>(
        &'a self,
        _channel: &'a str,
        _payload: &'a [u8],
    ) -> BoxFuture<'a, Result<(), CoordinationError>> {
        Box::pin(async move { Ok(()) })
    }

    fn increment_with_ttl<'a>(
        &'a self,
        key: &'a str,
        _ttl: Duration,
    ) -> BoxFuture<'a, Result<u64, CoordinationError>> {
        Box::pin(async move {
            let mut counts = self.counts.lock().expect("lock");
            match counts.iter_mut().find(|(candidate, _)| candidate == key) {
                Some((_, count)) => {
                    *count += 1;
                    Ok(*count)
                }
                None => {
                    counts.push((key.to_owned(), 1));
                    Ok(1)
                }
            }
        })
    }
}

/// Minimal participant store: enough to observe that an approval created one.
#[derive(Clone, Default)]
struct FakeParticipants {
    participants: Arc<Mutex<Vec<mahfel_domain::Participant>>>,
}

impl crate::ports::ParticipantStore for FakeParticipants {
    fn get(
        &self,
        id: mahfel_domain::ParticipantId,
    ) -> BoxFuture<'_, Result<Option<mahfel_domain::Participant>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .participants
                .lock()
                .expect("lock")
                .iter()
                .find(|participant| participant.id == id)
                .cloned())
        })
    }

    fn get_active(
        &self,
        session_id: SessionId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<mahfel_domain::Participant>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .participants
                .lock()
                .expect("lock")
                .iter()
                .find(|participant| {
                    participant.session_id == session_id
                        && participant.actor_id == actor_id
                        && participant.status == mahfel_domain::ParticipantStatus::Active
                })
                .cloned())
        })
    }

    fn insert<'a>(
        &'a self,
        participant: &'a mahfel_domain::Participant,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut participants = self.participants.lock().expect("lock");
            if participants.iter().any(|existing| {
                existing.session_id == participant.session_id
                    && existing.actor_id == participant.actor_id
                    && existing.status == mahfel_domain::ParticipantStatus::Active
            }) {
                return Err(StoreError::Conflict("already active".to_owned()));
            }
            participants.push(participant.clone());
            Ok(())
        })
    }

    fn save<'a>(
        &'a self,
        participant: &'a mahfel_domain::Participant,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut participants = self.participants.lock().expect("lock");
            if let Some(slot) = participants
                .iter_mut()
                .find(|existing| existing.id == participant.id)
            {
                *slot = participant.clone();
            }
            Ok(())
        })
    }

    fn list_for_session(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Vec<mahfel_domain::Participant>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .participants
                .lock()
                .expect("lock")
                .iter()
                .filter(|participant| participant.session_id == session_id)
                .cloned()
                .collect())
        })
    }
}

/// Minimal participation store: an approved join must open a clock.
#[derive(Clone, Default)]
struct FakeParticipation {
    intervals: Arc<Mutex<Vec<mahfel_domain::ParticipationInterval>>>,
}

impl crate::ports::ParticipationStore for FakeParticipation {
    fn open_interval<'a>(
        &'a self,
        interval: &'a mahfel_domain::ParticipationInterval,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            self.intervals.lock().expect("lock").push(interval.clone());
            Ok(())
        })
    }

    fn close_open_interval(
        &self,
        _participant_id: mahfel_domain::ParticipantId,
        _ended_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<Option<mahfel_domain::IntervalId>, StoreError>> {
        Box::pin(async move { Ok(None) })
    }

    fn close_open_intervals_for_session(
        &self,
        _session_id: SessionId,
        _ended_at: OffsetDateTime,
    ) -> BoxFuture<'_, Result<u64, StoreError>> {
        Box::pin(async move { Ok(0) })
    }

    fn get(
        &self,
        id: mahfel_domain::IntervalId,
    ) -> BoxFuture<'_, Result<Option<mahfel_domain::ParticipationInterval>, StoreError>> {
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
        participant_id: mahfel_domain::ParticipantId,
    ) -> BoxFuture<'_, Result<Vec<mahfel_domain::ParticipationInterval>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .intervals
                .lock()
                .expect("lock")
                .iter()
                .filter(|interval| interval.participant_id == participant_id)
                .cloned()
                .collect())
        })
    }

    fn list_for_session(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Vec<mahfel_domain::ParticipationInterval>, StoreError>> {
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
        _original: &'a mahfel_domain::ParticipationInterval,
        _replacement: &'a mahfel_domain::ParticipationInterval,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move { Ok(()) })
    }
}

struct Fixture {
    rooms: FakeRooms,
    sessions: FakeSessions,
    invites: FakeInvites,
    admissions: FakeAdmissions,
    coordination: FakeCoordination,
    owner: ActorId,
    room: Room,
    session: Session,
}

fn fixture() -> Fixture {
    let rooms = FakeRooms::default();
    let owner = ActorId::new();
    let mut room = rooms.seed_room(owner, RoomVisibility::InviteOnly);
    room.set_admission(AdmissionPolicy::Lobby, OffsetDateTime::now_utc())
        .expect("policy");
    let sessions = FakeSessions::default();
    let mut session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    session.start(OffsetDateTime::now_utc()).expect("start");
    sessions.seed(session.clone());
    Fixture {
        rooms,
        sessions,
        invites: FakeInvites::default(),
        admissions: FakeAdmissions::default(),
        coordination: FakeCoordination::default(),
        owner,
        room,
        session,
    }
}

fn link_for(f: &Fixture, target: InviteTarget, expiry_days: i64) -> InviteLink {
    let link = InviteLink::issue(
        InviteCode::generate(),
        target,
        f.owner,
        expiry_days,
        OffsetDateTime::now_utc(),
    )
    .expect("issue");
    f.invites.seed(link.clone());
    link
}

async fn redeem(f: &Fixture, actor: ActorId, code: &str) -> Result<Redeemed, RedeemError> {
    redeem_invite_link(
        &f.invites,
        &f.admissions,
        &f.sessions,
        &f.rooms,
        &f.coordination,
        &AttemptLimiter::default(),
        RedeemInviteLink {
            actor,
            code: code.to_owned(),
            source_address: Some("203.0.113.9".to_owned()),
        },
        OffsetDateTime::now_utc(),
    )
    .await
}

fn assert_not_redeemable(result: &Result<Redeemed, RedeemError>, context: &str) {
    assert!(
        matches!(result, Err(RedeemError::NotRedeemable(_))),
        "{context}, got {result:?}"
    );
}

#[tokio::test]
async fn a_newcomer_with_a_room_link_waits_for_a_decision() {
    let f = fixture();
    let link = link_for(&f, InviteTarget::Room(f.room.id), 7);
    let newcomer = ActorId::new();

    let redeemed = redeem(&f, newcomer, link.code.as_str())
        .await
        .expect("redeem");
    assert!(redeemed.is_waiting(), "a non-member must not be admitted");
    assert_eq!(redeemed.target(), InviteTarget::Room(f.room.id));

    let pending = f.admissions.all();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].origin, AdmissionOrigin::InviteLink);
    assert_eq!(pending[0].target, AdmissionTarget::Room(f.room.id));
    assert!(
        !f.rooms.is_member(f.room.id, newcomer),
        "waiting must not grant membership"
    );
}

#[tokio::test]
async fn a_newcomer_with_a_session_link_waits_scoped_to_that_session() {
    let f = fixture();
    let link = link_for(&f, InviteTarget::Session(f.session.id), 7);

    let redeemed = redeem(&f, ActorId::new(), link.code.as_str())
        .await
        .expect("redeem");
    assert!(redeemed.is_waiting());
    assert_eq!(
        f.admissions.all()[0].target,
        AdmissionTarget::Session(f.session.id),
        "a session link is scoped to its session"
    );
}

#[tokio::test]
async fn the_code_is_normalized_before_lookup() {
    let f = fixture();
    let link = link_for(&f, InviteTarget::Room(f.room.id), 7);
    let pasted = link.code.format().to_uppercase();

    let redeemed = redeem(&f, ActorId::new(), &pasted).await.expect("redeem");
    assert!(redeemed.is_waiting(), "a mixed-case paste must resolve");
}

#[tokio::test]
async fn an_existing_member_skips_the_lobby() {
    let f = fixture();
    let member = ActorId::new();
    f.rooms.add_member(f.room.id, member, RoomRole::Member);
    let link = link_for(&f, InviteTarget::Session(f.session.id), 7);

    let redeemed = redeem(&f, member, link.code.as_str())
        .await
        .expect("redeem");
    assert!(
        !redeemed.is_waiting(),
        "an explicit member never waits (AC 1)"
    );
    assert!(
        f.admissions.all().is_empty(),
        "no request is queued for a member"
    );
}

#[tokio::test]
async fn every_unusable_code_returns_the_same_error() {
    let f = fixture();
    let now = OffsetDateTime::now_utc();

    // Build each unusable link without leaving a live one in the store, so the
    // revoked case really is revoked when it is redeemed.
    let revoked = InviteLink::issue(
        InviteCode::generate(),
        InviteTarget::Room(f.room.id),
        f.owner,
        7,
        now,
    )
    .expect("issue");
    let mut revoked = revoked;
    revoked.revoke(now).expect("revoke");
    f.invites.seed(revoked.clone());

    let expired = InviteLink::issue(
        InviteCode::generate(),
        InviteTarget::Room(f.room.id),
        f.owner,
        1,
        now - time::Duration::days(2),
    )
    .expect("issue");
    f.invites.seed(expired.clone());

    let unknown = InviteCode::generate().format();
    let results = [
        redeem(&f, ActorId::new(), &unknown).await,
        redeem(&f, ActorId::new(), revoked.code.as_str()).await,
        redeem(&f, ActorId::new(), expired.code.as_str()).await,
        redeem(&f, ActorId::new(), "not-a-code").await,
    ];
    for result in &results {
        assert_not_redeemable(result, "every unusable code must fail identically");
    }
    let rendered: Vec<String> = results
        .iter()
        .map(|result| match result {
            Err(error) => error.to_string(),
            Ok(_) => panic!("expected a failure"),
        })
        .collect();
    assert!(
        rendered.windows(2).all(|pair| pair[0] == pair[1]),
        "unknown, revoked, expired, and malformed must read identically: {rendered:?}"
    );
}

#[tokio::test]
async fn a_link_whose_target_is_gone_fails_closed() {
    let f = fixture();
    // TASK-129 defers reaping, so a row can outlive its target. It must not be
    // redeemable, and the failure must not be distinguishable.
    let link = link_for(&f, InviteTarget::Session(SessionId::new()), 7);

    let result = redeem(&f, ActorId::new(), link.code.as_str()).await;
    assert_not_redeemable(&result, "a dangling target must fail closed");
}

#[tokio::test]
async fn an_archived_rooms_link_is_dead() {
    let f = fixture();
    let link = link_for(&f, InviteTarget::Room(f.room.id), 7);
    f.rooms.archive(f.room.id);

    let result = redeem(&f, ActorId::new(), link.code.as_str()).await;
    assert_not_redeemable(&result, "an archived room is closed to new members");
}

#[tokio::test]
async fn attempts_are_rate_limited_before_the_code_is_looked_at() {
    let f = fixture();
    let actor = ActorId::new();

    for _ in 0..MAX_ATTEMPTS_PER_PRINCIPAL {
        let _ = redeem(&f, actor, "abcdefghij").await;
    }

    // A genuinely valid code must now be refused, because the limit is checked
    // first and a limited attempt is indistinguishable from a bad code.
    let link = link_for(&f, InviteTarget::Room(f.room.id), 7);
    let result = redeem(&f, actor, link.code.as_str()).await;
    assert_not_redeemable(&result, "a limited attempt must look like a bad code");
    assert!(
        f.admissions.all().is_empty(),
        "a limited attempt must not queue anything"
    );
}

#[tokio::test]
async fn approving_a_room_link_grants_membership_and_never_management() {
    let f = fixture();
    let link = link_for(&f, InviteTarget::Room(f.room.id), 7);
    let newcomer = ActorId::new();
    redeem(&f, newcomer, link.code.as_str())
        .await
        .expect("redeem");

    let grant = crate::admission::admit(
        &FakeParticipants::default(),
        &FakeParticipation::default(),
        &f.admissions,
        &f.sessions,
        &f.rooms,
        AdmissionDecision {
            actor: f.owner,
            target: AdmissionTarget::Room(f.room.id),
            subject: newcomer,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("admit");

    assert!(
        grant.participant().is_none(),
        "a room approval grants membership, not session participation (AC 3)"
    );
    let membership = f
        .rooms
        .get_member(f.room.id, newcomer)
        .await
        .expect("get")
        .expect("member");
    assert_eq!(
        membership.role,
        RoomRole::Member,
        "an invite link must never grant the manager role"
    );
}

#[tokio::test]
async fn approving_a_session_link_grants_no_membership() {
    let f = fixture();
    let link = link_for(&f, InviteTarget::Session(f.session.id), 7);
    let newcomer = ActorId::new();
    redeem(&f, newcomer, link.code.as_str())
        .await
        .expect("redeem");

    let grant = crate::admission::admit(
        &FakeParticipants::default(),
        &FakeParticipation::default(),
        &f.admissions,
        &f.sessions,
        &f.rooms,
        AdmissionDecision {
            actor: f.owner,
            target: AdmissionTarget::Session(f.session.id),
            subject: newcomer,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("admit");

    assert!(
        grant.participant().is_some(),
        "a session approval admits to that session (AC 2)"
    );
    assert!(
        !f.rooms.is_member(f.room.id, newcomer),
        "a session link must create no durable room membership"
    );
}

#[tokio::test]
async fn a_room_link_is_approvable_with_no_live_session() {
    let f = fixture();
    f.sessions.end(f.session.id);
    let link = link_for(&f, InviteTarget::Room(f.room.id), 7);
    let newcomer = ActorId::new();
    let redeemed = redeem(&f, newcomer, link.code.as_str())
        .await
        .expect("redeem");
    assert!(redeemed.is_waiting());

    let grant = crate::admission::admit(
        &FakeParticipants::default(),
        &FakeParticipation::default(),
        &f.admissions,
        &f.sessions,
        &f.rooms,
        AdmissionDecision {
            actor: f.owner,
            target: AdmissionTarget::Room(f.room.id),
            subject: newcomer,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("a room approval needs no live session");

    assert!(grant.participant().is_none());
    assert!(f.rooms.is_member(f.room.id, newcomer));
}

#[tokio::test]
async fn a_session_link_approval_still_requires_a_live_session() {
    let f = fixture();
    let link = link_for(&f, InviteTarget::Session(f.session.id), 7);
    let newcomer = ActorId::new();
    redeem(&f, newcomer, link.code.as_str())
        .await
        .expect("redeem");
    f.sessions.end(f.session.id);

    let result = crate::admission::admit(
        &FakeParticipants::default(),
        &FakeParticipation::default(),
        &f.admissions,
        &f.sessions,
        &f.rooms,
        AdmissionDecision {
            actor: f.owner,
            target: AdmissionTarget::Session(f.session.id),
            subject: newcomer,
        },
        OffsetDateTime::now_utc(),
    )
    .await;
    assert!(
        result.is_err(),
        "attending a session that is not live is not meaningful"
    );
}

#[tokio::test]
async fn a_room_scoped_request_survives_the_session_sweep() {
    let f = fixture();
    let link = link_for(&f, InviteTarget::Room(f.room.id), 7);
    let newcomer = ActorId::new();
    redeem(&f, newcomer, link.code.as_str())
        .await
        .expect("redeem");

    // Ending a session retires session-scoped requests only: a room request
    // asks to join something that outlives the session.
    let expired = f
        .admissions
        .expire_pending_for_session(f.session.id, OffsetDateTime::now_utc())
        .await
        .expect("sweep");
    assert_eq!(
        expired, 0,
        "a room request must not be swept by a session ending"
    );
    assert!(f.admissions.all()[0].is_pending());
}

#[tokio::test]
async fn a_member_who_cannot_manage_cannot_decide() {
    let f = fixture();
    let link = link_for(&f, InviteTarget::Room(f.room.id), 7);
    let newcomer = ActorId::new();
    redeem(&f, newcomer, link.code.as_str())
        .await
        .expect("redeem");

    // A member can see the room but does not manage it.
    let member = ActorId::new();
    f.rooms.add_member(f.room.id, member, RoomRole::Member);

    let result = crate::admission::admit(
        &FakeParticipants::default(),
        &FakeParticipation::default(),
        &f.admissions,
        &f.sessions,
        &f.rooms,
        AdmissionDecision {
            actor: member,
            target: AdmissionTarget::Room(f.room.id),
            subject: newcomer,
        },
        OffsetDateTime::now_utc(),
    )
    .await;
    assert!(
        result.is_err(),
        "a plain member must not approve an invite (AC 4)"
    );
    assert!(
        !f.rooms.is_member(f.room.id, newcomer),
        "a denied decision grants nothing"
    );
}

#[tokio::test]
async fn rejecting_terminates_the_waiting_state_and_grants_nothing() {
    let f = fixture();
    let link = link_for(&f, InviteTarget::Room(f.room.id), 7);
    let newcomer = ActorId::new();
    redeem(&f, newcomer, link.code.as_str())
        .await
        .expect("redeem");

    crate::admission::reject(
        &f.admissions,
        &f.sessions,
        &f.rooms,
        AdmissionDecision {
            actor: f.owner,
            target: AdmissionTarget::Room(f.room.id),
            subject: newcomer,
        },
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("reject");

    assert!(!f.rooms.is_member(f.room.id, newcomer));
    assert!(
        f.admissions
            .all()
            .iter()
            .all(|request| !request.is_pending()),
        "rejection ends the waiting state (AC 4)"
    );
}
