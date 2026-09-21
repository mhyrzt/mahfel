use super::{
    AppState, api_router,
    auth::{
        AuthState, AuthenticationError, RequestAuthenticator, ServiceKeyAuthenticator,
        StoreServiceKeyAuthenticator, TokenAuthentication,
    },
    cors_layer,
    service_principals::{ServicePrincipalAdmin, StoreServicePrincipalAdmin},
    ws::{ParticipationStores, WsConfig, WsMetrics, WsState},
};
use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Method, Request, StatusCode, header},
};
use futures_util::{SinkExt, StreamExt};
use mahfel_application::policy::ServerCapabilityResolver;
use mahfel_application::ports::{
    AttributeError, BoxFuture, Capability, CapabilityContext, CapabilityResolver, CapabilitySet,
    CoordinationError, CoordinationStore, InviteStore, ParticipantStore, ParticipationStore,
    Principal, ProvisionedIdentity, RoomStore, ServicePrincipalStore, SessionStore, StoreError,
    UserAttributeWriter,
};
use mahfel_application::service_principals::{CreateServicePrincipal, create_service_principal};
use mahfel_application::users::{IdentityProvisioner, SyncIdentity, SyncIdentityError};
use mahfel_auth::AuthenticatedIdentity;
use mahfel_auth::{TokenVerifier, TokenVerifierConfig};
use mahfel_config::TurnSettings;
use mahfel_domain::{
    ActorId, ActorKind, DisplayName, Identity, IntervalId, InviteCode, InviteLink, InviteLinkId,
    InviteLinkStatus, InviteTarget, InviteTargetKind, Issuer, Participant, ParticipantId,
    ParticipationInterval, Room, RoomId, RoomMember, RoomRole, RoomTitle, RoomVisibility,
    ServicePrincipal, ServicePrincipalStatus, Session, SessionId, SessionStatus, Subject, User,
    UserStatus,
};
use mahfel_media_client::GrpcMediaGateway;
use prometheus::Encoder;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use time::OffsetDateTime;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::watch,
    task::JoinHandle,
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message as ClientWsMessage,
};
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone, Default)]
struct FakeRoomStore {
    inner: Arc<Mutex<FakeRoomStoreInner>>,
}

#[derive(Default)]
struct FakeRoomStoreInner {
    rooms: Vec<Room>,
    members: Vec<RoomMember>,
}

impl FakeRoomStore {
    fn seed_room(&self, title: &str, owner: ActorId, visibility: RoomVisibility) -> Room {
        let now = OffsetDateTime::now_utc();
        let room = Room::create(
            RoomTitle::new(title.to_owned()).expect("valid title"),
            owner,
            visibility,
            now,
        );
        let manager = RoomMember::new(room.id, owner, RoomRole::Manager, now);
        let mut inner = self.inner.lock().expect("fake store lock");
        inner.rooms.push(room.clone());
        inner.members.push(manager);
        room
    }

    fn add_raw_member(&self, room_id: RoomId, user_id: ActorId, role: RoomRole) {
        let mut inner = self.inner.lock().expect("fake store lock");
        inner.members.push(RoomMember::new(
            room_id,
            user_id,
            role,
            OffsetDateTime::now_utc(),
        ));
    }
}

impl RoomStore for FakeRoomStore {
    fn get(&self, id: RoomId) -> BoxFuture<'_, Result<Option<Room>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .inner
                .lock()
                .expect("fake store lock")
                .rooms
                .iter()
                .find(|room| room.id == id)
                .cloned())
        })
    }

    fn insert<'a>(&'a self, room: &'a Room) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            self.inner
                .lock()
                .expect("fake store lock")
                .rooms
                .push(room.clone());
            Ok(())
        })
    }

    fn save<'a>(&'a self, room: &'a Room) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut inner = self.inner.lock().expect("fake store lock");
            let slot = inner
                .rooms
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
            let mut inner = self.inner.lock().expect("fake store lock");
            inner.rooms.push(room.clone());
            inner.members.push(manager.clone());
            Ok(())
        })
    }

    fn list_for_member(
        &self,
        user_id: ActorId,
        limit: u64,
        offset: u64,
    ) -> BoxFuture<'_, Result<Vec<Room>, StoreError>> {
        Box::pin(async move {
            let inner = self.inner.lock().expect("fake store lock");
            let mut rooms: Vec<Room> = inner
                .rooms
                .iter()
                .filter(|room| {
                    inner
                        .members
                        .iter()
                        .any(|member| member.room_id == room.id && member.actor_id == user_id)
                })
                .cloned()
                .collect();
            rooms.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then_with(|| left.id.as_uuid().cmp(&right.id.as_uuid()))
            });
            Ok(rooms
                .into_iter()
                .skip(offset as usize)
                .take(limit as usize)
                .collect())
        })
    }

    fn get_member(
        &self,
        room_id: RoomId,
        user_id: ActorId,
    ) -> BoxFuture<'_, Result<Option<RoomMember>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .inner
                .lock()
                .expect("fake store lock")
                .members
                .iter()
                .find(|member| member.room_id == room_id && member.actor_id == user_id)
                .cloned())
        })
    }

    fn list_members(&self, room_id: RoomId) -> BoxFuture<'_, Result<Vec<RoomMember>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .inner
                .lock()
                .expect("fake store lock")
                .members
                .iter()
                .filter(|member| member.room_id == room_id)
                .cloned()
                .collect())
        })
    }

    fn add_member<'a>(&'a self, member: &'a RoomMember) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut inner = self.inner.lock().expect("fake store lock");
            if inner.members.iter().any(|existing| {
                existing.room_id == member.room_id && existing.actor_id == member.actor_id
            }) {
                return Err(StoreError::Conflict("already a member".to_owned()));
            }
            inner.members.push(member.clone());
            Ok(())
        })
    }

    fn remove_member(
        &self,
        room_id: RoomId,
        user_id: ActorId,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            let mut inner = self.inner.lock().expect("fake store lock");
            let managers = inner
                .members
                .iter()
                .filter(|member| member.room_id == room_id && member.role == RoomRole::Manager)
                .count();
            let before = inner.members.len();
            inner.members.retain(|member| {
                let is_target = member.room_id == room_id && member.actor_id == user_id;
                !is_target || (member.role == RoomRole::Manager && managers <= 1)
            });
            Ok(inner.members.len() < before)
        })
    }
}

/// In-memory [`SessionStore`] for transport tests.
///
/// Mirrors the database's one-live-session-per-room partial unique index so a
/// second concurrent start is observable as a conflict at the transport.
#[derive(Clone, Default)]
struct FakeSessionStore {
    sessions: Arc<Mutex<Vec<Session>>>,
}

impl FakeSessionStore {
    fn seed(&self, session: Session) {
        self.sessions.lock().expect("fake store lock").push(session);
    }

    fn has_live(&self, room_id: RoomId, except: Option<SessionId>) -> bool {
        self.sessions
            .lock()
            .expect("fake store lock")
            .iter()
            .any(|s| {
                s.room_id == room_id && s.status == SessionStatus::Live && except != Some(s.id)
            })
    }
}

impl SessionStore for FakeSessionStore {
    fn get(&self, id: SessionId) -> BoxFuture<'_, Result<Option<Session>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .sessions
                .lock()
                .expect("fake store lock")
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
            self.sessions
                .lock()
                .expect("fake store lock")
                .push(session.clone());
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
            let mut sessions = self.sessions.lock().expect("fake store lock");
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
                .expect("fake store lock")
                .iter()
                .filter(|session| session.room_id == room_id)
                .cloned()
                .collect())
        })
    }
}

/// In-memory [`ServicePrincipalStore`] for transport tests.
#[derive(Clone, Default)]
struct FakeServicePrincipalStore {
    inner: Arc<Mutex<FakeServicePrincipalInner>>,
}

#[derive(Default)]
struct FakeServicePrincipalInner {
    principals: Vec<ServicePrincipal>,
    active_keys: std::collections::HashMap<String, ActorId>,
}

impl ServicePrincipalStore for FakeServicePrincipalStore {
    fn create<'a>(
        &'a self,
        principal: &'a ServicePrincipal,
        api_key_hash: &'a str,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut inner = self.inner.lock().expect("fake store lock");
            if inner
                .principals
                .iter()
                .any(|other| other.name == principal.name)
            {
                return Err(StoreError::Conflict("duplicate name".to_owned()));
            }
            inner.principals.push(principal.clone());
            inner
                .active_keys
                .insert(api_key_hash.to_owned(), principal.id);
            Ok(())
        })
    }

    fn get(&self, id: ActorId) -> BoxFuture<'_, Result<Option<ServicePrincipal>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .inner
                .lock()
                .expect("fake store lock")
                .principals
                .iter()
                .find(|principal| principal.id == id)
                .cloned())
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<ServicePrincipal>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .inner
                .lock()
                .expect("fake store lock")
                .principals
                .clone())
        })
    }

    fn revoke(&self, id: ActorId) -> BoxFuture<'_, Result<(), StoreError>> {
        Box::pin(async move {
            let mut inner = self.inner.lock().expect("fake store lock");
            let Some(principal) = inner.principals.iter_mut().find(|p| p.id == id) else {
                return Err(StoreError::NotFound(id.to_string()));
            };
            principal.status = ServicePrincipalStatus::Revoked;
            inner.active_keys.retain(|_, owner| *owner != id);
            Ok(())
        })
    }

    fn rotate_key<'a>(
        &'a self,
        id: ActorId,
        api_key_hash: &'a str,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut inner = self.inner.lock().expect("fake store lock");
            let Some(principal) = inner.principals.iter().find(|p| p.id == id) else {
                return Err(StoreError::NotFound(id.to_string()));
            };
            if principal.status != ServicePrincipalStatus::Active {
                return Err(StoreError::NotFound(id.to_string()));
            }
            inner.active_keys.retain(|_, owner| *owner != id);
            inner.active_keys.insert(api_key_hash.to_owned(), id);
            Ok(())
        })
    }

    fn find_active_by_key_hash<'a>(
        &'a self,
        api_key_hash: &'a str,
    ) -> BoxFuture<'a, Result<Option<ServicePrincipal>, StoreError>> {
        Box::pin(async move {
            let inner = self.inner.lock().expect("fake store lock");
            let Some(id) = inner.active_keys.get(api_key_hash).copied() else {
                return Ok(None);
            };
            Ok(inner
                .principals
                .iter()
                .find(|principal| {
                    principal.id == id && principal.status == ServicePrincipalStatus::Active
                })
                .cloned())
        })
    }
}

/// Build the auth/admin seams over one shared fake service principal store.
fn service_seams(
    store: FakeServicePrincipalStore,
) -> (
    Arc<dyn ServiceKeyAuthenticator>,
    Arc<dyn ServicePrincipalAdmin>,
) {
    (
        Arc::new(StoreServiceKeyAuthenticator::new(store.clone())),
        Arc::new(StoreServicePrincipalAdmin::new(store)),
    )
}

/// Test double: yields a fixed principal, or rejects when none is configured.
#[derive(Clone)]
struct StubAuthenticator {
    principal: Option<Principal>,
    provisioned: bool,
}

impl StubAuthenticator {
    fn new(principal: Option<Principal>) -> Self {
        Self {
            principal,
            provisioned: true,
        }
    }

    fn unprovisioned(principal: Option<Principal>) -> Self {
        Self {
            principal,
            provisioned: false,
        }
    }
}

impl RequestAuthenticator for StubAuthenticator {
    fn verify<'a>(
        &'a self,
        _token: Option<&'a str>,
    ) -> BoxFuture<'a, Result<AuthenticatedIdentity, AuthenticationError>> {
        let result = self
            .principal
            .clone()
            .map(|principal| AuthenticatedIdentity {
                subject: principal.actor_id.to_string(),
                issuer: "https://auth.test/realms/rastai".to_owned(),
                app_user_id: self.provisioned.then(|| principal.actor_id.to_string()),
                app_principal_kind: Some(
                    match principal.kind {
                        ActorKind::User => "user",
                        ActorKind::Service => "service",
                    }
                    .to_owned(),
                ),
                email: principal.email.clone(),
                email_verified: principal.email_verified,
                username: principal.username.clone(),
                display_name: principal.display_name.clone(),
                avatar_url: principal.avatar_url.clone(),
                roles: principal.roles.clone(),
            })
            .ok_or(AuthenticationError::MissingToken);
        Box::pin(async move { result })
    }
}

/// Test double that reflects the claims back into a local user.
struct StubProvisioner;

impl IdentityProvisioner for StubProvisioner {
    fn provision<'a>(
        &'a self,
        command: SyncIdentity,
    ) -> BoxFuture<'a, Result<ProvisionedIdentity, SyncIdentityError>> {
        let SyncIdentity {
            issuer,
            subject,
            email_verified,
            ..
        } = command;
        let user_id = Uuid::parse_str(&subject)
            .map(ActorId::from_uuid)
            .unwrap_or_default();
        let now = OffsetDateTime::now_utc();
        let user = User {
            id: user_id,
            display_name: DisplayName::new("Test User".to_owned()).expect("valid name"),
            status: UserStatus::Active,
            avatar_url: None,
            created_at: now,
            updated_at: now,
        };
        let identity = Identity::create(
            user_id,
            Issuer::new(issuer).expect("valid issuer"),
            Subject::new(subject).expect("valid subject"),
            None,
            None,
            email_verified,
            None,
            now,
        );
        Box::pin(async move { Ok(ProvisionedIdentity { user, identity }) })
    }
}

/// Test double that accepts every identity link.
struct StubLinker;

impl UserAttributeWriter for StubLinker {
    fn set_user_attribute<'a>(
        &'a self,
        _user_id: &'a str,
        _name: &'a str,
        _value: &'a str,
    ) -> BoxFuture<'a, Result<(), AttributeError>> {
        Box::pin(async { Ok(()) })
    }
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
            .expect("fake store lock")
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
                .expect("fake store lock")
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
                .expect("fake store lock")
                .push(participant.clone());
            Ok(())
        })
    }

    fn save<'a>(&'a self, participant: &'a Participant) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut participants = self.participants.lock().expect("fake store lock");
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
                .expect("fake store lock")
                .iter()
                .filter(|p| p.session_id == session_id)
                .cloned()
                .collect())
        })
    }
}

/// In-memory [`ParticipationStore`] mirroring the database invariants that
/// matter to the transport: at most one open interval per participant, and
/// corrections that retain the superseded row.
#[derive(Clone, Default)]
struct FakeParticipationStore {
    intervals: Arc<Mutex<Vec<ParticipationInterval>>>,
}

impl FakeParticipationStore {
    fn seed(&self, interval: ParticipationInterval) {
        self.intervals
            .lock()
            .expect("fake store lock")
            .push(interval);
    }

    fn list_for_participant(&self, participant_id: ParticipantId) -> Vec<ParticipationInterval> {
        self.intervals
            .lock()
            .expect("fake store lock")
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
            let mut intervals = self.intervals.lock().expect("fake store lock");
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
        ended_at: time::OffsetDateTime,
    ) -> BoxFuture<'_, Result<Option<IntervalId>, StoreError>> {
        Box::pin(async move {
            let mut intervals = self.intervals.lock().expect("fake store lock");
            let found = intervals
                .iter_mut()
                .find(|interval| interval.participant_id == participant_id && interval.is_open());
            match found {
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
        ended_at: time::OffsetDateTime,
    ) -> BoxFuture<'_, Result<u64, StoreError>> {
        Box::pin(async move {
            let mut intervals = self.intervals.lock().expect("fake store lock");
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
                .expect("fake store lock")
                .iter()
                .find(|interval| interval.id == id)
                .cloned())
        })
    }

    fn list_for_participant(
        &self,
        participant_id: ParticipantId,
    ) -> BoxFuture<'_, Result<Vec<ParticipationInterval>, StoreError>> {
        Box::pin(async move { Ok(self.list_for_participant(participant_id)) })
    }

    fn list_for_session(
        &self,
        session_id: SessionId,
    ) -> BoxFuture<'_, Result<Vec<ParticipationInterval>, StoreError>> {
        Box::pin(async move {
            Ok(self
                .intervals
                .lock()
                .expect("fake store lock")
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
            let mut intervals = self.intervals.lock().expect("fake store lock");
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

/// Counting coordination double for the invite rate limiter.
///
/// Mirrors the Redis adapter's contract: the first increment creates the key,
/// later ones count up. Expiry is not modelled because the limiter's window
/// behavior is tested where the policy lives; here it only has to count.
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
            let mut counts = self.counts.lock().expect("fake coordination lock");
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

/// In-memory invite links, with the same uniqueness rule the unique index
/// enforces.
#[derive(Clone, Default)]
struct FakeInviteStore {
    links: Arc<Mutex<Vec<InviteLink>>>,
}

impl FakeInviteStore {
    fn seed(&self, link: InviteLink) {
        self.links.lock().expect("fake store lock").push(link);
    }

    fn links(&self) -> Vec<InviteLink> {
        self.links.lock().expect("fake store lock").clone()
    }
}

impl InviteStore for FakeInviteStore {
    fn insert<'a>(&'a self, link: &'a InviteLink) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut links = self.links.lock().expect("fake store lock");
            if links.iter().any(|existing| existing.code == link.code) {
                return Err(StoreError::Conflict("code taken".to_owned()));
            }
            links.push(link.clone());
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
                .expect("fake store lock")
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
                .expect("fake store lock")
                .iter()
                .find(|link| link.id == id)
                .cloned())
        })
    }

    fn list_for_target(
        &self,
        target_kind: InviteTargetKind,
        target_id: Uuid,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<InviteLink>, StoreError>> {
        Box::pin(async move {
            let mut links: Vec<InviteLink> = self
                .links
                .lock()
                .expect("fake store lock")
                .iter()
                .filter(|link| link.target.kind() == target_kind && link.target.id() == target_id)
                .cloned()
                .collect();
            links.sort_by_key(|link| std::cmp::Reverse(link.created_at));
            links.truncate(limit as usize);
            Ok(links)
        })
    }

    fn revoke_if_active(
        &self,
        id: InviteLinkId,
        revoked_at: time::OffsetDateTime,
    ) -> BoxFuture<'_, Result<bool, StoreError>> {
        Box::pin(async move {
            let mut links = self.links.lock().expect("fake store lock");
            let Some(link) = links.iter_mut().find(|link| link.id == id) else {
                return Ok(false);
            };
            if link.status == InviteLinkStatus::Revoked {
                return Ok(false);
            }
            link.status = InviteLinkStatus::Revoked;
            link.revoked_at = Some(revoked_at);
            Ok(true)
        })
    }

    fn rotate<'a>(
        &'a self,
        previous: InviteLinkId,
        revoked_at: time::OffsetDateTime,
        replacement: &'a InviteLink,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let mut links = self.links.lock().expect("fake store lock");
            if links.iter().any(|link| link.code == replacement.code) {
                return Err(StoreError::Conflict("code taken".to_owned()));
            }
            let Some(index) = links.iter().position(|link| link.id == previous) else {
                return Err(StoreError::NotFound("no such link".to_owned()));
            };
            if links[index].status == InviteLinkStatus::Revoked {
                return Err(StoreError::Conflict("already revoked".to_owned()));
            }
            links[index].status = InviteLinkStatus::Revoked;
            links[index].revoked_at = Some(revoked_at);
            links.push(replacement.clone());
            Ok(())
        })
    }
}

fn ws_state(principal: Option<Principal>) -> WsState {
    let (_shutdown_tx, shutdown_rx) = watch::channel(false);
    let metrics = WsMetrics::register(&prometheus::Registry::new()).expect("register ws metrics");
    WsState::new(
        Arc::new(StubAuthenticator::new(principal)),
        WsConfig::default(),
        metrics,
        Arc::new(GrpcMediaGateway::lazy("http://localhost:50051".to_owned()).expect("media")),
        ParticipationStores {
            participants: Arc::new(FakeParticipantStore::default()),
            participation: Arc::new(FakeParticipationStore::default()),
            sessions: Arc::new(FakeSessionStore::default()),
            rooms: Arc::new(FakeRoomStore::default()),
        },
        Arc::new(FakeCoordination::default()),
        shutdown_rx,
    )
}

fn state(principal: Option<Principal>, authorizer: Arc<dyn CapabilityResolver>) -> AppState {
    state_with(principal, authorizer, true)
}

fn state_unprovisioned(
    principal: Option<Principal>,
    authorizer: Arc<dyn CapabilityResolver>,
) -> AppState {
    state_with(principal, authorizer, false)
}

fn state_with(
    principal: Option<Principal>,
    authorizer: Arc<dyn CapabilityResolver>,
    provisioned: bool,
) -> AppState {
    let realtime = ws_state(principal.clone());
    let (service_auth, service_principals) = service_seams(FakeServicePrincipalStore::default());
    AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(authorizer),
        authentication: Arc::new(if provisioned {
            StubAuthenticator::new(principal)
        } else {
            StubAuthenticator::unprovisioned(principal)
        }),
        service_auth,
        service_principals,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: Arc::new(FakeRoomStore::default()),
        sessions: Arc::new(FakeSessionStore::default()),
        participation: Arc::new(FakeParticipationStore::default()),
        invites: Arc::new(FakeInviteStore::default()),
        code_index: Arc::new(FakeCoordination::default()),
        realtime,
        turn: turn_settings(),
    }
}

fn principal_with(value: &str) -> Principal {
    Principal {
        actor_id: ActorId::from_uuid(Uuid::parse_str(value).expect("valid uuid")),
        kind: ActorKind::User,
        display_name: Some("Test User".to_owned()),
        avatar_url: None,
        email: Some("test@mahfel.local".to_owned()),
        email_verified: true,
        username: Some("test".to_owned()),
        roles: Vec::new(),
    }
}

fn principal() -> Principal {
    principal_with("00000000-0000-0000-0000-000000000009")
}

fn other_principal() -> Principal {
    principal_with("00000000-0000-0000-0000-000000000010")
}

/// A user principal for an actor the test already created, so authorization
/// resolves against that actor's membership.
fn principal_for(actor_id: ActorId) -> Principal {
    Principal {
        actor_id,
        ..principal()
    }
}

fn turn_settings() -> TurnSettings {
    TurnSettings {
        static_secret: "test-turn-secret".to_owned(),
        realm: "mahfel.local".to_owned(),
        uris: vec!["turn:127.0.0.1:3478?transport=udp".to_owned()],
        ttl_seconds: 300,
    }
}

fn app() -> Router {
    api_router(state(None, Arc::new(AllowAll)))
}

fn authenticated_app() -> Router {
    api_router(state(Some(principal()), Arc::new(AllowAll)))
}

fn authenticated_room_app(principal: Principal, store: FakeRoomStore) -> Router {
    let realtime = ws_state(Some(principal.clone()));
    let (service_auth, service_principals) = service_seams(FakeServicePrincipalStore::default());
    api_router(AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(AllowAll)),
        authentication: Arc::new(StubAuthenticator::new(Some(principal))),
        service_auth,
        service_principals,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: Arc::new(store),
        sessions: Arc::new(FakeSessionStore::default()),
        participation: Arc::new(FakeParticipationStore::default()),
        invites: Arc::new(FakeInviteStore::default()),
        code_index: Arc::new(FakeCoordination::default()),
        realtime,
        turn: turn_settings(),
    })
}

/// Always permits. Tests that are not about authorization use this so policy
/// behaviour is not coupled to every unrelated assertion.
struct AllowAll;

impl CapabilityResolver for AllowAll {
    fn resolve(&self, _principal: &Principal, _context: CapabilityContext) -> CapabilitySet {
        CapabilitySet::new(vec![
            Capability::ManageSystem,
            Capability::CreateRoom,
            Capability::ViewOwnIdentity,
            Capability::Join,
            Capability::PublishMedia,
            Capability::ShareScreen,
            Capability::MuteOthers,
            Capability::RemoveParticipant,
            Capability::AdmitParticipant,
            Capability::ManageSession,
        ])
    }
}

struct DenyAll;

impl CapabilityResolver for DenyAll {
    fn resolve(&self, _principal: &Principal, _context: CapabilityContext) -> CapabilitySet {
        CapabilitySet::none()
    }
}

async fn response_body_string(resp: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .expect("body readable");
    String::from_utf8(bytes.to_vec()).expect("utf8")
}

async fn response_json(resp: axum::response::Response) -> serde_json::Value {
    let body = response_body_string(resp).await;
    serde_json::from_str(&body).expect("valid json")
}

fn error_code(json: &serde_json::Value) -> Option<&str> {
    json.get("error")
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_str())
}

#[tokio::test]
async fn status_success_has_request_id_and_security_headers() {
    let req = Request::builder()
        .uri("/v1/status")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key("x-request-id"));
    assert_eq!(
        resp.headers()
            .get("x-content-type-options")
            .map(|v| v.to_str().expect("str")),
        Some("nosniff")
    );
    assert_eq!(
        resp.headers()
            .get("x-frame-options")
            .map(|v| v.to_str().expect("str")),
        Some("DENY")
    );
    assert!(resp.headers().contains_key("referrer-policy"));

    let json = response_json(resp).await;
    assert_eq!(
        json.get("service").and_then(|v| v.as_str()),
        Some("mahfel-api-test")
    );
    assert!(json.get("version").is_some());
    assert!(json.get("node_id").is_some());
}

#[tokio::test]
async fn request_id_is_propagated_when_client_provides() {
    let req = Request::builder()
        .uri("/v1/health")
        .header("x-request-id", "test-id-123")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok()),
        Some("test-id-123")
    );
}

#[tokio::test]
async fn invalid_json_maps_to_422_envelope_not_500() {
    let req = Request::builder()
        .uri("/v1/echo")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{ not json }"))
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(resp.headers().contains_key("x-request-id"));
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("unprocessable_entity"));
}

#[tokio::test]
async fn validation_error_maps_to_422_with_envelope() {
    let req = Request::builder()
        .uri("/v1/echo")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"message":""}"#))
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("unprocessable_entity"));
}

#[tokio::test]
async fn pagination_defaults_and_limits() {
    let app = app();

    let req = Request::builder()
        .uri("/v1/echo")
        .body(Body::empty())
        .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    let pagination = json.get("pagination").expect("pagination meta");
    assert_eq!(pagination.get("page").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(
        pagination.get("per_page").and_then(|v| v.as_u64()),
        Some(20)
    );
    assert!(json.get("data").is_some());

    let req = Request::builder()
        .uri("/v1/echo?per_page=999")
        .body(Body::empty())
        .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response_json(resp).await.get("error").is_some());

    let req = Request::builder()
        .uri("/v1/echo?page=0")
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unknown_route_returns_404_envelope() {
    let req = Request::builder()
        .uri("/v1/unknown")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert!(resp.headers().contains_key("x-request-id"));
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("not_found"));
}

#[tokio::test]
async fn unversioned_unknown_route_returns_404_envelope() {
    let req = Request::builder()
        .uri("/not-an-api-route")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("not_found"));
}

#[tokio::test]
async fn identity_requires_authentication() {
    let req = Request::builder()
        .uri("/v1/auth/me")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert!(resp.headers().contains_key("x-request-id"));
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("unauthorized"));
}

#[tokio::test]
async fn identity_returns_the_authenticated_principal() {
    let req = Request::builder()
        .uri("/v1/auth/me")
        .body(Body::empty())
        .expect("request");
    let resp = authenticated_app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    assert_eq!(
        json.get("user_id").and_then(|v| v.as_str()),
        Some("00000000-0000-0000-0000-000000000009")
    );
    assert_eq!(
        json.get("email").and_then(|v| v.as_str()),
        Some("test@mahfel.local")
    );
    assert_eq!(json.get("email_verified"), Some(&serde_json::json!(true)));
}

#[tokio::test]
async fn identity_is_denied_when_policy_denies() {
    let app = api_router(state(Some(principal()), Arc::new(DenyAll)));
    let req = Request::builder()
        .uri("/v1/auth/me")
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("forbidden"));
}

#[tokio::test]
async fn identity_rejects_a_token_without_a_linked_user() {
    // A verified token that has not been provisioned must not reach handlers.
    let app = api_router(state_unprovisioned(Some(principal()), Arc::new(AllowAll)));
    let req = Request::builder()
        .uri("/v1/auth/me")
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("forbidden"));
}

#[tokio::test]
async fn provision_returns_the_application_profile() {
    // Provisioning must remain reachable before the user is linked.
    let app = api_router(state_unprovisioned(Some(principal()), Arc::new(AllowAll)));
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/provision")
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    assert_eq!(
        json.get("user_id").and_then(|v| v.as_str()),
        Some("00000000-0000-0000-0000-000000000009")
    );
    assert_eq!(
        json.get("email").and_then(|v| v.as_str()),
        Some("test@mahfel.local")
    );
}

#[tokio::test]
async fn admin_ping_allows_admin_and_denies_member_and_moderator() {
    let admin = Principal {
        actor_id: principal().actor_id,
        kind: mahfel_domain::ActorKind::User,
        display_name: Some("Admin".to_owned()),
        avatar_url: None,
        email: Some("admin@mahfel.local".to_owned()),
        email_verified: true,
        username: Some("admin".to_owned()),
        roles: vec!["admin".to_owned()],
    };
    let moderator = Principal {
        roles: vec!["moderator".to_owned()],
        ..principal()
    };
    let member = principal();

    let admin_app = api_router(state(Some(admin), Arc::new(ServerCapabilityResolver)));
    let req = Request::builder()
        .uri("/v1/admin/ping")
        .body(Body::empty())
        .expect("request");
    let resp = admin_app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);

    for (principal, label) in [(moderator, "moderator"), (member, "member")] {
        let app = api_router(state(Some(principal), Arc::new(ServerCapabilityResolver)));
        let req = Request::builder()
            .uri("/v1/admin/ping")
            .body(Body::empty())
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "{label} must be denied for admin ping"
        );
    }
}

#[tokio::test]
async fn cors_preflight_allows_configured_origin_and_exposes_request_id() {
    let origin = HeaderValue::from_static("http://localhost:3000");
    let app = api_router(state(None, Arc::new(AllowAll))).layer(cors_layer(origin));

    let req = Request::builder()
        .uri("/v1/status")
        .method(Method::OPTIONS)
        .header(header::ORIGIN, "http://localhost:3000")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
        .body(Body::empty())
        .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    assert_eq!(
        resp.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|v| v.to_str().ok()),
        Some("http://localhost:3000")
    );
    assert!(
        resp.headers()
            .get(header::ACCESS_CONTROL_ALLOW_METHODS)
            .is_some(),
        "preflight advertises allowed methods"
    );

    let req = Request::builder()
        .uri("/v1/status")
        .header(header::ORIGIN, "http://localhost:3000")
        .body(Body::empty())
        .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    let exposed = resp
        .headers()
        .get(header::ACCESS_CONTROL_EXPOSE_HEADERS)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert!(
        exposed.contains("x-request-id"),
        "exposes request id: {exposed}"
    );

    let req = Request::builder()
        .uri("/v1/status")
        .method(Method::OPTIONS)
        .header(header::ORIGIN, "http://evil.example")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert!(
        resp.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none(),
        "unconfigured origin must not be allowed"
    );
}

#[tokio::test]
async fn pagination_slice_is_correct() {
    let req = Request::builder()
        .uri("/v1/echo?page=2&per_page=2")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    let data = json.get("data").and_then(|d| d.as_array()).expect("array");
    assert_eq!(data.len(), 2);
    assert_eq!(
        data[0].get("message").and_then(|m| m.as_str()),
        Some("gamma")
    );
}

async fn send_json(
    app: &Router,
    method: Method,
    uri: &str,
    body: &str,
) -> axum::response::Response {
    let req = Request::builder()
        .uri(uri)
        .method(method)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .expect("request");
    app.clone().oneshot(req).await.expect("response")
}

#[tokio::test]
async fn room_is_created_and_read_by_its_creator() {
    let app = authenticated_room_app(principal(), FakeRoomStore::default());
    let resp = send_json(&app, Method::POST, "/v1/rooms", r#"{"title":"Algebra"}"#).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = response_json(resp).await;
    assert_eq!(
        json.get("visibility").and_then(|v| v.as_str()),
        Some("invite_only")
    );
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("room id")
        .to_owned();

    let req = Request::builder()
        .uri(format!("/v1/rooms/{id}"))
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    assert_eq!(json.get("title").and_then(|v| v.as_str()), Some("Algebra"));
}

#[tokio::test]
async fn room_requires_authentication() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Algebra", principal().actor_id, RoomVisibility::Public);
    let req = Request::builder()
        .uri(format!("/v1/rooms/{}", room.id))
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn room_is_404_for_a_non_member() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Algebra", principal().actor_id, RoomVisibility::Public);
    let app = authenticated_room_app(other_principal(), store);
    let req = Request::builder()
        .uri(format!("/v1/rooms/{}", room.id))
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("not_found"));
}

#[tokio::test]
async fn room_list_returns_only_joined_rooms() {
    let store = FakeRoomStore::default();
    store.seed_room("Owned", principal().actor_id, RoomVisibility::InviteOnly);
    store.seed_room("Other", other_principal().actor_id, RoomVisibility::Public);
    let app = authenticated_room_app(principal(), store);
    let req = Request::builder()
        .uri("/v1/rooms")
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    let data = json.get("data").and_then(|d| d.as_array()).expect("array");
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].get("title").and_then(|v| v.as_str()), Some("Owned"));
}

#[tokio::test]
async fn joining_a_public_room_is_idempotent() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Open", other_principal().actor_id, RoomVisibility::Public);
    let app = authenticated_room_app(principal(), store.clone());
    let uri = format!("/v1/rooms/{}/join", room.id);

    let req = Request::builder()
        .uri(&uri)
        .method(Method::POST)
        .body(Body::empty())
        .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);

    let req = Request::builder()
        .uri(&uri)
        .method(Method::POST)
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        store
            .get_member(room.id, principal().actor_id)
            .await
            .expect("store")
            .is_some()
    );
}

#[tokio::test]
async fn joining_an_invite_only_room_is_404() {
    let store = FakeRoomStore::default();
    let room = store.seed_room(
        "Private",
        other_principal().actor_id,
        RoomVisibility::InviteOnly,
    );
    let app = authenticated_room_app(principal(), store);
    let req = Request::builder()
        .uri(format!("/v1/rooms/{}/join", room.id))
        .method(Method::POST)
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn manager_can_add_and_remove_a_member() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let target = other_principal().actor_id;
    let app = authenticated_room_app(principal(), store.clone());

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/members", room.id),
        &format!(r#"{{"user_id":"{target}"}}"#),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    assert!(
        store
            .get_member(room.id, target)
            .await
            .expect("store")
            .is_some()
    );

    let req = Request::builder()
        .uri(format!("/v1/rooms/{}/members/{target}", room.id))
        .method(Method::DELETE)
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    assert!(
        store
            .get_member(room.id, target)
            .await
            .expect("store")
            .is_none()
    );
}

#[tokio::test]
async fn non_manager_member_cannot_add_a_member() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    store.add_raw_member(room.id, other_principal().actor_id, RoomRole::Member);
    let app = authenticated_room_app(other_principal(), store);
    let third = principal_with("00000000-0000-0000-0000-000000000011").actor_id;

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/members", room.id),
        &format!(r#"{{"user_id":"{third}"}}"#),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn non_member_cannot_add_a_member() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let app = authenticated_room_app(other_principal(), store);
    let target = principal_with("00000000-0000-0000-0000-000000000011").actor_id;

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/members", room.id),
        &format!(r#"{{"user_id":"{target}"}}"#),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn last_manager_cannot_be_removed() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let owner = principal().actor_id;
    let app = authenticated_room_app(principal(), store.clone());

    let req = Request::builder()
        .uri(format!("/v1/rooms/{}/members/{owner}", room.id))
        .method(Method::DELETE)
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert!(
        store
            .get_member(room.id, owner)
            .await
            .expect("store")
            .is_some()
    );
}

#[tokio::test]
async fn visibility_change_preserves_members() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let member = other_principal().actor_id;
    store.add_raw_member(room.id, member, RoomRole::Member);
    let app = authenticated_room_app(principal(), store.clone());

    let resp = send_json(
        &app,
        Method::PATCH,
        &format!("/v1/rooms/{}", room.id),
        r#"{"visibility":"public"}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    assert_eq!(
        json.get("visibility").and_then(|v| v.as_str()),
        Some("public")
    );
    assert!(
        store
            .get_member(room.id, member)
            .await
            .expect("store")
            .is_some()
    );
}

#[tokio::test]
async fn empty_room_title_is_422() {
    let app = authenticated_room_app(principal(), FakeRoomStore::default());
    let resp = send_json(&app, Method::POST, "/v1/rooms", r#"{"title":"  "}"#).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn manager_can_archive_a_room() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let app = authenticated_room_app(principal(), store);

    let req = Request::builder()
        .uri(format!("/v1/rooms/{}/archive", room.id))
        .method(Method::POST)
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    assert_eq!(
        json.get("status").and_then(|v| v.as_str()),
        Some("archived")
    );
}

#[tokio::test]
async fn non_manager_cannot_archive_a_room() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    store.add_raw_member(room.id, other_principal().actor_id, RoomRole::Member);
    let app = authenticated_room_app(other_principal(), store);

    let req = Request::builder()
        .uri(format!("/v1/rooms/{}/archive", room.id))
        .method(Method::POST)
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn joining_an_archived_room_is_conflict() {
    let store = FakeRoomStore::default();
    let room = store.seed_room("Open", other_principal().actor_id, RoomVisibility::Public);

    let owner_app = authenticated_room_app(other_principal(), store.clone());
    let req = Request::builder()
        .uri(format!("/v1/rooms/{}/archive", room.id))
        .method(Method::POST)
        .body(Body::empty())
        .expect("request");
    let resp = owner_app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);

    let joiner_app = authenticated_room_app(principal(), store);
    let req = Request::builder()
        .uri(format!("/v1/rooms/{}/join", room.id))
        .method(Method::POST)
        .body(Body::empty())
        .expect("request");
    let resp = joiner_app.oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("conflict"));
}

#[tokio::test]
async fn malformed_room_id_maps_to_error_envelope() {
    let app = authenticated_room_app(principal(), FakeRoomStore::default());
    let req = Request::builder()
        .uri("/v1/rooms/not-a-uuid")
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("invalid_input"));
}

// --- Session lifecycle transport tests (TASK-26) ---

/// Router with a supplied room and session store, optionally authenticated.
fn session_app(
    principal: Option<Principal>,
    rooms: FakeRoomStore,
    sessions: FakeSessionStore,
) -> Router {
    let realtime = ws_state(principal.clone());
    let (service_auth, service_principals) = service_seams(FakeServicePrincipalStore::default());
    api_router(AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(AllowAll)),
        authentication: Arc::new(StubAuthenticator::new(principal)),
        service_auth,
        service_principals,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: Arc::new(rooms),
        sessions: Arc::new(sessions),
        participation: Arc::new(FakeParticipationStore::default()),
        invites: Arc::new(FakeInviteStore::default()),
        code_index: Arc::new(FakeCoordination::default()),
        realtime,
        turn: turn_settings(),
    })
}

#[tokio::test]
async fn manager_can_schedule_start_and_end_a_session() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let app = session_app(Some(principal()), rooms, FakeSessionStore::default());

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/sessions", room.id),
        "{}",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = response_json(resp).await;
    assert_eq!(
        json.get("status").and_then(|v| v.as_str()),
        Some("scheduled")
    );
    let session_id = json
        .get("id")
        .and_then(|v| v.as_str())
        .expect("session id")
        .to_owned();

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/sessions/{session_id}/start"),
        "",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("live"));

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/sessions/{session_id}/end"),
        "",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("ended"));

    // The manager can read it back and list it.
    let req = Request::builder()
        .uri(format!("/v1/sessions/{session_id}"))
        .body(Body::empty())
        .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);

    let req = Request::builder()
        .uri(format!("/v1/rooms/{}/sessions", room.id))
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    assert_eq!(json.as_array().map(Vec::len), Some(1));
}

#[tokio::test]
async fn non_member_cannot_schedule_a_session() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room(
        "Algebra",
        other_principal().actor_id,
        RoomVisibility::InviteOnly,
    );
    let app = session_app(Some(principal()), rooms, FakeSessionStore::default());

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/sessions", room.id),
        "{}",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn member_cannot_schedule_a_session() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    rooms.add_raw_member(room.id, other_principal().actor_id, RoomRole::Member);
    let app = session_app(Some(other_principal()), rooms, FakeSessionStore::default());

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/sessions", room.id),
        "{}",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn invalid_session_transition_returns_conflict() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let sessions = FakeSessionStore::default();
    let session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.seed(session.clone());
    let app = session_app(Some(principal()), rooms, sessions);

    // Ending a scheduled session is invalid.
    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/sessions/{}/end", session.id),
        "",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("conflict"));
}

#[tokio::test]
async fn second_live_session_returns_conflict() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let sessions = FakeSessionStore::default();
    let first = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    let second = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.seed(first.clone());
    sessions.seed(second.clone());
    let app = session_app(Some(principal()), rooms, sessions);

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/sessions/{}/start", first.id),
        "",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/sessions/{}/start", second.id),
        "",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn session_routes_require_authentication() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let sessions = FakeSessionStore::default();
    let session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.seed(session.clone());
    let app = session_app(None, rooms, sessions);

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/sessions/{}/start", session.id),
        "",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// --- Session capability introspection (TASK-124) ---

/// Router whose authorization uses the real policy, so the capability endpoint
/// answers from resolved membership rather than a test fake.
fn policy_session_app(
    principal: Option<Principal>,
    rooms: FakeRoomStore,
    sessions: FakeSessionStore,
) -> Router {
    let realtime = ws_state(principal.clone());
    let (service_auth, service_principals) = service_seams(FakeServicePrincipalStore::default());
    api_router(AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(ServerCapabilityResolver)),
        authentication: Arc::new(StubAuthenticator::new(principal)),
        service_auth,
        service_principals,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: Arc::new(rooms),
        sessions: Arc::new(sessions),
        participation: Arc::new(FakeParticipationStore::default()),
        invites: Arc::new(FakeInviteStore::default()),
        code_index: Arc::new(FakeCoordination::default()),
        realtime,
        turn: turn_settings(),
    })
}

fn capabilities_of(json: &serde_json::Value) -> Vec<String> {
    json.get("capabilities")
        .and_then(|value| value.as_array())
        .expect("capabilities array")
        .iter()
        .map(|value| value.as_str().expect("capability name").to_owned())
        .collect()
}

#[tokio::test]
async fn manager_introspects_the_full_room_capability_set() {
    let rooms = FakeRoomStore::default();
    let manager = principal();
    let room = rooms.seed_room("Algebra", manager.actor_id, RoomVisibility::InviteOnly);
    let sessions = FakeSessionStore::default();
    let session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.seed(session.clone());
    let app = policy_session_app(Some(manager), rooms, sessions);

    let resp = send_json(
        &app,
        Method::GET,
        &format!("/v1/sessions/{}/capabilities", session.id),
        "",
    )
    .await;

    assert_eq!(resp.status(), StatusCode::OK);
    let capabilities = capabilities_of(&response_json(resp).await);
    assert!(capabilities.contains(&"manage_session".to_owned()));
    assert!(capabilities.contains(&"mute_others".to_owned()));
    assert!(
        !capabilities.contains(&"manage_system".to_owned()),
        "a room manager is not a system administrator"
    );
}

#[tokio::test]
async fn member_introspection_excludes_moderation() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let member = other_principal();
    rooms.add_raw_member(room.id, member.actor_id, RoomRole::Member);
    let sessions = FakeSessionStore::default();
    let session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.seed(session.clone());
    let app = policy_session_app(Some(member), rooms, sessions);

    let resp = send_json(
        &app,
        Method::GET,
        &format!("/v1/sessions/{}/capabilities", session.id),
        "",
    )
    .await;

    let status = resp.status();
    let body = response_body_string(resp).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let capabilities =
        capabilities_of(&serde_json::from_str(&body).expect("capabilities response is JSON"));
    assert!(capabilities.contains(&"publish_media".to_owned()));
    assert!(!capabilities.contains(&"manage_session".to_owned()));
    assert!(!capabilities.contains(&"remove_participant".to_owned()));
}

#[tokio::test]
async fn a_non_member_cannot_introspect_a_private_session() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let sessions = FakeSessionStore::default();
    let session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.seed(session.clone());
    let app = policy_session_app(Some(other_principal()), rooms, sessions);

    let resp = send_json(
        &app,
        Method::GET,
        &format!("/v1/sessions/{}/capabilities", session.id),
        "",
    )
    .await;

    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "a caller without access must not learn the session exists"
    );
}

type ClientSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct WsHarness {
    addr: SocketAddr,
    shutdown: watch::Sender<bool>,
    registry: prometheus::Registry,
    _task: JoinHandle<std::io::Result<()>>,
}

async fn serve_ws(principal: Option<Principal>, config: WsConfig) -> WsHarness {
    let registry = prometheus::Registry::new();
    let metrics = WsMetrics::register(&registry).expect("register ws metrics");
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let realtime = WsState::new(
        Arc::new(StubAuthenticator::new(principal.clone())),
        config,
        metrics,
        Arc::new(GrpcMediaGateway::lazy("http://localhost:50051".to_owned()).expect("media")),
        ParticipationStores {
            participants: Arc::new(FakeParticipantStore::default()),
            participation: Arc::new(FakeParticipationStore::default()),
            sessions: Arc::new(FakeSessionStore::default()),
            rooms: Arc::new(FakeRoomStore::default()),
        },
        Arc::new(FakeCoordination::default()),
        shutdown_rx.clone(),
    );
    let (service_auth, service_principals) = service_seams(FakeServicePrincipalStore::default());
    let state = AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(AllowAll)),
        authentication: Arc::new(StubAuthenticator::new(principal)),
        service_auth,
        service_principals,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: Arc::new(FakeRoomStore::default()),
        sessions: Arc::new(FakeSessionStore::default()),
        participation: Arc::new(FakeParticipationStore::default()),
        invites: Arc::new(FakeInviteStore::default()),
        code_index: Arc::new(FakeCoordination::default()),
        realtime,
        turn: turn_settings(),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("listener address");
    let task = tokio::spawn(mahfel_telemetry::serve_http(
        listener,
        api_router(state),
        shutdown_rx,
    ));
    WsHarness {
        addr,
        shutdown: shutdown_tx,
        registry,
        _task: task,
    }
}

async fn connect(addr: SocketAddr) -> ClientSocket {
    let (socket, _) = connect_async(format!("ws://{addr}/v1/ws"))
        .await
        .expect("connect");
    socket
}

async fn send(socket: &mut ClientSocket, frame: &str) {
    socket
        .send(ClientWsMessage::text(frame.to_owned()))
        .await
        .expect("send frame");
}

async fn next(socket: &mut ClientSocket) -> ClientWsMessage {
    loop {
        match socket
            .next()
            .await
            .expect("frame present")
            .expect("frame ok")
        {
            ClientWsMessage::Ping(_) | ClientWsMessage::Pong(_) => continue,
            frame => return frame,
        }
    }
}

async fn authenticate(socket: &mut ClientSocket) {
    send(socket, r#"{"v":1,"type":"auth"}"#).await;
    assert!(
        next(socket)
            .await
            .to_text()
            .expect("text")
            .contains("welcome"),
        "authentication must be acknowledged with a welcome frame"
    );
}

async fn wait_for_metric(registry: &prometheus::Registry, needle: &str) {
    let encoder = prometheus::TextEncoder::new();
    for _ in 0..300 {
        let mut buffer = Vec::new();
        encoder
            .encode(&registry.gather(), &mut buffer)
            .expect("encode metrics");
        if String::from_utf8_lossy(&buffer).contains(needle) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("metric not observed: {needle}");
}

// --- Real-token WS helpers (TASK-121) ---

const WS_ISSUER_AUDIENCE: &str = "mahfel-api";
const WS_KEY_ID: &str = "test-key";
const WS_TEST_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDbmVO19GITNOuj\nVpKeftrusY3jZjR+z1kOVwNwkM0avBU82CEGyPozUaQ4/eOtjOB6QOeTzayew3KC\nVAH6Oj4nivrrUrF6iATLWDFyXqjnDC3lbNV85bVRdhNeozNXYHYxxrhgExoxiJwK\n1FOKFXLNsOg+Hu2/UHlqNu0Y47Of7S+7lCJ+xCbxvRF8o6JSVimaHdP1y4ESwBZF\nqzyxBKogPBWpRNbBehLVCTbdrIVunot/ONEFiNuposcKjh0GDJB4Ng/4WSZenAGC\njCoO26y5N27NdARErCYj047qlUteCTUBm/bs8EdsqVqmx3ODdOhsqkqiHw8htSW+\nhL9vS7FnAgMBAAECggEAU1Om9BA7dtZJ47zh/cpdAN3W3UbW/JdpT4us/nWznkVo\nOaI5Rf3VEswT6JxOXHhsZXdjaC5PJh87wEBoT67yDscp10mVls0uJIremGta4/E3\nKStXBwGS/8oyJ2L+BFXSA6lIJQyaj9kqfOF8oZTp7ysFtSy/Qi0jJS6aYStl6XS+\n6KGrSLmArXIuzNZ3/byOunBWQWbfsxSNitRiFXDqsxfpA0cbsj4adb3gpQl919jB\naZ5Fi8VzYPqP7kOE0APuI85SmWxPhX1vXbbmwqCA/JWPzXnnxU0KR+k1Npsqbg6a\nITvO0VLYHY56fQi1tFY77JepT2XQwbDPUipy1T3foQKBgQD2hQrkZ/VkupH/6NZ6\nTw8n0IMQOLyxomEQ4Lj0Ugizj7KbGyEN8w+6CSU84y9VclR/trW7laa3FMr++bC8\nGY7wkONZFsdsbtLkLSlU9LILPKdAOcbBQDJaOZip5ll07JU+TMTZQpE0KfHDDCjH\n4RmMKzlslpBmaY3WWDR/rz8EpQKBgQDkC0CwVO+vp8JPjze3BRViJytU1G9cKcpd\nO2t1W/KINYIQ0eqkBV5l7wLq1ht8JmMRpa3gLHHxdIoxouKzM3kSqvPGkoUTO2Cj\nf5HPnCBJmk7paKD6Y4FBA4SsRqYlp+uKfmzLlleQ7hIkEK8H5ur8CTzsTV4mhX08\nrKrex9MkGwKBgCETPp/gltsZ8SbNvaERZG4qGrudcSvAJpfWHtqg+wvkdsk6vZea\nifyi530g2iPzwASWphFyBblAr++/3oeOQlBikC5O0RjqUZ9Ns+LzJcQ+Mg568gDR\ntS/H1ITPJCd86pU+RZcEyGigTBCoW/xdZkkZL44zMVKcTOx6TyjmgWUFAoGBAOHo\nT9Ye3i7Z9sFP9HanX46TlPZ9WDAp6iHACm0BgFr0/vtgqJyCAQjj5LT1E9dCQ+Qo\nh/5KqTn0JC6V2RSDSk528fXmElSB/vMZHj5XwRiI9+F71/UT++iG7rinaFJnbgwN\nwANS1T5qyafWqz4dwO2azTKeVkW8+drQhpkZxY2/AoGBAO8wvaUA+yeaUof6/QFx\niwz7RIX6QlztIcV6rQBXmbsS4zHdQ1aCMCMctatqozztewJk0qa8dRRm8KNjNbLr\nnCg+0ezO+qF2AI+7WAaaicScxpJkW7WUXFVW3wIdKyFStuAXzY7JIH6biTxgUU1W\n4fZV7NStTEG5uoe0960K55+T\n-----END PRIVATE KEY-----";
const WS_MODULUS: &str = "25lTtfRiEzTro1aSnn7a7rGN42Y0fs9ZDlcDcJDNGrwVPNghBsj6M1GkOP3jrYzgekDnk82snsNyglQB-jo-J4r661KxeogEy1gxcl6o5wwt5WzVfOW1UXYTXqMzV2B2Mca4YBMaMYicCtRTihVyzbDoPh7tv1B5ajbtGOOzn-0vu5QifsQm8b0RfKOiUlYpmh3T9cuBEsAWRas8sQSqIDwVqUTWwXoS1Qk23ayFbp6LfzjRBYjbqaLHCo4dBgyQeDYP-FkmXpwBgowqDtusuTduzXQERKwmI9OO6pVLXgk1AZv27PBHbKlapsdzg3TobKpKoh8PIbUlvoS_b0uxZw";

struct RealIssuer {
    url: String,
    _task: JoinHandle<()>,
}

async fn start_real_issuer() -> RealIssuer {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind issuer");
    let addr = listener.local_addr().expect("issuer addr");
    let url = format!("http://{addr}");
    let discovery_url = url.clone();
    let discovery = axum::Router::new().route(
        "/.well-known/openid-configuration",
        axum::routing::get(move || {
            let url = discovery_url.clone();
            async move {
                axum::Json(serde_json::json!({
                    "issuer": url,
                    "jwks_uri": format!("{url}/jwks"),
                }))
            }
        }),
    );
    let jwks = axum::Router::new().route(
        "/jwks",
        axum::routing::get(move || async move {
            axum::Json(serde_json::json!({
                "keys": [{
                    "kty": "RSA",
                    "kid": WS_KEY_ID,
                    "alg": "RS256",
                    "use": "sig",
                    "n": WS_MODULUS,
                    "e": "AQAB",
                }]
            }))
        }),
    );
    let app = discovery.merge(jwks);
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    RealIssuer { url, _task: task }
}

fn ws_mint(claims: serde_json::Value) -> String {
    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some(WS_KEY_ID.to_owned());
    let key =
        jsonwebtoken::EncodingKey::from_rsa_pem(WS_TEST_KEY_PEM.as_bytes()).expect("test key");
    jsonwebtoken::encode(&header, &claims, &key).expect("mint")
}

fn ws_claims(issuer: &str, user_id: &str, offset_seconds: i64) -> serde_json::Value {
    let now = time::OffsetDateTime::now_utc().unix_timestamp() + offset_seconds;
    serde_json::json!({
        "iss": issuer,
        "aud": WS_ISSUER_AUDIENCE,
        "sub": user_id,
        "app_user_id": user_id,
        "email": "test@mahfel.local",
        "email_verified": true,
        "preferred_username": "test",
        "name": "Test User",
        "iat": now,
        "nbf": now,
        "exp": now + 300,
    })
}

fn ws_claims_unprovisioned(issuer: &str, offset_seconds: i64) -> serde_json::Value {
    let now = time::OffsetDateTime::now_utc().unix_timestamp() + offset_seconds;
    serde_json::json!({
        "iss": issuer,
        "aud": WS_ISSUER_AUDIENCE,
        "sub": "00000000-0000-0000-0000-000000000099",
        "email": "test@mahfel.local",
        "email_verified": true,
        "preferred_username": "test",
        "name": "Test User",
        "iat": now,
        "nbf": now,
        "exp": now + 300,
    })
}

fn ws_verifier(issuer: &RealIssuer) -> TokenVerifier {
    let mut cfg = TokenVerifierConfig::new(issuer.url.clone(), WS_ISSUER_AUDIENCE);
    cfg.leeway = Duration::from_secs(1);
    TokenVerifier::new(cfg)
}

async fn serve_ws_with_verifier(verifier: TokenVerifier, config: WsConfig) -> WsHarness {
    let registry = prometheus::Registry::new();
    let metrics = WsMetrics::register(&registry).expect("register ws metrics");
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let auth: Arc<dyn RequestAuthenticator> = Arc::new(TokenAuthentication::new(verifier));
    let realtime = WsState::new(
        Arc::clone(&auth),
        config,
        metrics,
        Arc::new(GrpcMediaGateway::lazy("http://localhost:50051".to_owned()).expect("media")),
        ParticipationStores {
            participants: Arc::new(FakeParticipantStore::default()),
            participation: Arc::new(FakeParticipationStore::default()),
            sessions: Arc::new(FakeSessionStore::default()),
            rooms: Arc::new(FakeRoomStore::default()),
        },
        Arc::new(FakeCoordination::default()),
        shutdown_rx.clone(),
    );
    let (service_auth, service_principals) = service_seams(FakeServicePrincipalStore::default());
    let state = AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(AllowAll)),
        authentication: auth,
        service_auth,
        service_principals,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: Arc::new(FakeRoomStore::default()),
        sessions: Arc::new(FakeSessionStore::default()),
        participation: Arc::new(FakeParticipationStore::default()),
        invites: Arc::new(FakeInviteStore::default()),
        code_index: Arc::new(FakeCoordination::default()),
        realtime,
        turn: turn_settings(),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let task = tokio::spawn(mahfel_telemetry::serve_http(
        listener,
        api_router(state),
        shutdown_rx,
    ));
    WsHarness {
        addr,
        shutdown: shutdown_tx,
        registry,
        _task: task,
    }
}

async fn connect_with_header(
    addr: SocketAddr,
    token: Option<&str>,
) -> Result<ClientSocket, tokio_tungstenite::tungstenite::Error> {
    use tokio_tungstenite::tungstenite::http::Request as HttpRequest;
    let uri = format!("ws://{addr}/v1/ws");
    let mut builder = HttpRequest::builder().uri(uri.as_str());
    if let Some(tok) = token {
        builder = builder.header(header::AUTHORIZATION.as_str(), format!("Bearer {tok}"));
    }
    // Tungstenite derives Sec-WebSocket-Key etc. if absent; we just need a valid
    // HTTP request with WS upgrade semantics. Passing empty body is fine for
    // IntoClientRequest.
    let req = builder
        .header(header::HOST.as_str(), addr.to_string())
        .header(header::CONNECTION.as_str(), "Upgrade")
        .header(header::UPGRADE.as_str(), "websocket")
        .header(header::SEC_WEBSOCKET_VERSION.as_str(), "13")
        .header(
            header::SEC_WEBSOCKET_KEY.as_str(),
            "dGhlIHNhbXBsZSBub25jZQ==",
        )
        .body(())
        .expect("ws request");
    let (socket, _) = connect_async(req).await?;
    Ok(socket)
}

fn ws_tamper_signature(token: &str) -> String {
    use base64::Engine;
    let mut parts: Vec<String> = token.split('.').map(|s| s.to_owned()).collect();
    let mut sig = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&parts[2])
        .expect("decode sig");
    sig[0] ^= 0x01;
    parts[2] = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&sig);
    parts.join(".")
}

#[tokio::test]
async fn websocket_authenticates_and_echoes() {
    let harness = serve_ws(Some(principal()), WsConfig::default()).await;
    let mut socket = connect(harness.addr).await;
    authenticate(&mut socket).await;

    send(&mut socket, r#"{"v":1,"type":"echo","text":"hi"}"#).await;
    let echo = next(&mut socket).await;
    assert!(echo.to_text().expect("text").contains(r#""type":"echo""#));
    assert!(echo.to_text().expect("text").contains(r#""text":"hi""#));
}

#[tokio::test]
async fn websocket_broadcasts_to_all_clients() {
    let harness = serve_ws(Some(principal()), WsConfig::default()).await;
    let mut first = connect(harness.addr).await;
    let mut second = connect(harness.addr).await;
    authenticate(&mut first).await;
    authenticate(&mut second).await;

    send(&mut first, r#"{"v":1,"type":"broadcast","text":"ping"}"#).await;

    let from_sender = next(&mut first).await;
    let from_peer = next(&mut second).await;
    assert!(
        from_sender
            .to_text()
            .expect("text")
            .contains(r#""type":"broadcast""#)
    );
    assert!(
        from_peer
            .to_text()
            .expect("text")
            .contains(r#""text":"ping""#)
    );
}

#[tokio::test]
async fn websocket_denies_unauthenticated_clients() {
    let harness = serve_ws(None, WsConfig::default()).await;
    let mut socket = connect(harness.addr).await;
    send(&mut socket, r#"{"v":1,"type":"auth"}"#).await;

    loop {
        match socket.next().await {
            Some(Ok(ClientWsMessage::Close(_))) | None | Some(Err(_)) => break,
            Some(Ok(_)) => continue,
        }
    }
    wait_for_metric(
        &harness.registry,
        r#"websocket_disconnects_total{reason="unauthorized"} 1"#,
    )
    .await;
}

#[tokio::test]
async fn websocket_disconnect_releases_capacity() {
    let harness = serve_ws(Some(principal()), WsConfig::default()).await;
    let mut socket = connect(harness.addr).await;
    authenticate(&mut socket).await;
    wait_for_metric(&harness.registry, "websocket_connections_active 1").await;

    drop(socket);
    wait_for_metric(&harness.registry, "websocket_connections_active 0").await;
}

#[tokio::test]
async fn websocket_closes_idle_connections() {
    let config = WsConfig {
        heartbeat_interval: Duration::from_millis(10),
        heartbeat_timeout: Duration::from_millis(40),
        ..WsConfig::default()
    };
    let harness = serve_ws(Some(principal()), config).await;
    let mut socket = connect(harness.addr).await;
    authenticate(&mut socket).await;

    // Without reading, the client never answers the server's pings.
    wait_for_metric(
        &harness.registry,
        r#"websocket_disconnects_total{reason="heartbeat_timeout"} 1"#,
    )
    .await;
}

#[tokio::test]
async fn websocket_shutdown_closes_connections() {
    let harness = serve_ws(Some(principal()), WsConfig::default()).await;
    let mut socket = connect(harness.addr).await;
    authenticate(&mut socket).await;

    harness.shutdown.send(true).expect("signal shutdown");
    let closed = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match socket.next().await {
                Some(Ok(ClientWsMessage::Close(_))) | None | Some(Err(_)) => break,
                Some(Ok(_)) => continue,
            }
        }
    })
    .await;
    assert!(
        closed.is_ok(),
        "server must close the connection on shutdown"
    );
}

#[test]
fn openapi_document_covers_every_rest_route() {
    use utoipa::OpenApi;

    let document = crate::http::routes::openapi::ApiDoc::openapi();
    for path in crate::http::routes::openapi::REST_PATH_TEMPLATES {
        assert!(
            document.paths.paths.contains_key(*path),
            "REST route missing from the OpenAPI document: {path}"
        );
    }
}

#[tokio::test]
async fn openapi_document_is_served() {
    let req = Request::builder()
        .uri("/v1/openapi.json")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    assert!(
        json.get("openapi")
            .and_then(|value| value.as_str())
            .is_some_and(|value| value.starts_with("3.")),
        "document must declare an OpenAPI 3.x version"
    );
    assert!(json.get("paths").is_some());
}

#[tokio::test]
async fn websocket_rejects_clients_at_capacity() {
    let config = WsConfig {
        max_connections: 1,
        ..WsConfig::default()
    };
    let harness = serve_ws(Some(principal()), config).await;
    let mut first = connect(harness.addr).await;
    authenticate(&mut first).await;

    let rejected = connect_async(format!("ws://{}/v1/ws", harness.addr)).await;
    assert!(rejected.is_err(), "second connection must be rejected");
    wait_for_metric(&harness.registry, "websocket_connections_rejected_total 1").await;
}

// Real-token WS integration tests (TASK-121)

#[tokio::test]
async fn ws_header_auth_with_valid_token_succeeds() {
    let issuer = start_real_issuer().await;
    let user_id = "00000000-0000-0000-0000-000000000021";
    let token = ws_mint(ws_claims(&issuer.url, user_id, 0));
    let harness = serve_ws_with_verifier(ws_verifier(&issuer), WsConfig::default()).await;
    let mut socket = connect_with_header(harness.addr, Some(&token))
        .await
        .expect("header auth must succeed");

    let welcome = tokio::time::timeout(Duration::from_secs(2), next(&mut socket))
        .await
        .expect("welcome timeout");
    assert!(
        welcome.to_text().expect("text").contains("welcome"),
        "header-auth must yield welcome without an auth frame"
    );
    send(
        &mut socket,
        r#"{"v":1,"type":"echo","text":"hello-header"}"#,
    )
    .await;
    let echo = tokio::time::timeout(Duration::from_secs(2), next(&mut socket))
        .await
        .expect("echo timeout");
    assert!(echo.to_text().expect("text").contains(r#""type":"echo""#));
    assert!(echo.to_text().expect("text").contains("hello-header"));
}

#[tokio::test]
async fn ws_header_auth_with_invalid_sig_rejected() {
    let issuer = start_real_issuer().await;
    let token = ws_tamper_signature(&ws_mint(ws_claims(
        &issuer.url,
        "00000000-0000-0000-0000-000000000022",
        0,
    )));
    let harness = serve_ws_with_verifier(ws_verifier(&issuer), WsConfig::default()).await;
    let result = connect_with_header(harness.addr, Some(&token)).await;
    assert!(
        result.is_err(),
        "invalid signature over header must be rejected"
    );
    if let Err(tokio_tungstenite::tungstenite::Error::Http(resp)) = result {
        assert_eq!(
            resp.status(),
            401,
            "invalid signature must map to 401 at upgrade"
        );
    }
}

#[tokio::test]
async fn ws_message_auth_with_valid_token_succeeds() {
    let issuer = start_real_issuer().await;
    let user_id = "00000000-0000-0000-0000-000000000023";
    let token = ws_mint(ws_claims(&issuer.url, user_id, 0));
    let harness = serve_ws_with_verifier(ws_verifier(&issuer), WsConfig::default()).await;
    let mut socket = connect(harness.addr).await;
    let frame = serde_json::json!({"v":1,"type":"auth","token": token}).to_string();
    send(&mut socket, &frame).await;
    let welcome = tokio::time::timeout(Duration::from_secs(2), next(&mut socket))
        .await
        .expect("welcome timeout");
    assert!(welcome.to_text().expect("text").contains("welcome"));
    send(&mut socket, r#"{"v":1,"type":"echo","text":"hello-msg"}"#).await;
    let echo = tokio::time::timeout(Duration::from_secs(2), next(&mut socket))
        .await
        .expect("echo timeout");
    assert!(echo.to_text().expect("text").contains("hello-msg"));
}

#[tokio::test]
async fn ws_message_auth_with_expired_token_rejected() {
    let issuer = start_real_issuer().await;
    let token = ws_mint(ws_claims(
        &issuer.url,
        "00000000-0000-0000-0000-000000000024",
        -600,
    ));
    let harness = serve_ws_with_verifier(ws_verifier(&issuer), WsConfig::default()).await;
    let mut socket = connect(harness.addr).await;
    let frame = serde_json::json!({"v":1,"type":"auth","token": token}).to_string();
    send(&mut socket, &frame).await;

    let mut closed = false;
    let mut code_ok = false;
    let deadline = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match socket.next().await {
                Some(Ok(ClientWsMessage::Close(frame))) => {
                    closed = true;
                    if let Some(f) = frame
                        && u16::from(f.code) == 4401
                    {
                        code_ok = true;
                    }
                    break;
                }
                Some(Ok(_)) => continue,
                Some(Err(_)) | None => {
                    closed = true;
                    break;
                }
            }
        }
    })
    .await;
    assert!(deadline.is_ok(), "must close within timeout");
    assert!(closed, "expired token must close connection");
    assert!(code_ok, "expired token must close with 4401");
    wait_for_metric(
        &harness.registry,
        r#"websocket_disconnects_total{reason="unauthorized"} 1"#,
    )
    .await;
}

#[tokio::test]
async fn ws_rejects_unprovisioned_token() {
    let issuer = start_real_issuer().await;
    let token = ws_mint(ws_claims_unprovisioned(&issuer.url, 0));
    let harness = serve_ws_with_verifier(ws_verifier(&issuer), WsConfig::default()).await;
    let mut socket = connect(harness.addr).await;
    let frame = serde_json::json!({"v":1,"type":"auth","token": token}).to_string();
    send(&mut socket, &frame).await;

    let mut closed = false;
    let mut code_ok = false;
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match socket.next().await {
                Some(Ok(ClientWsMessage::Close(frame))) => {
                    closed = true;
                    if let Some(f) = frame
                        && u16::from(f.code) == 4401
                    {
                        code_ok = true;
                    }
                    break;
                }
                Some(Ok(_)) => continue,
                Some(Err(_)) | None => {
                    closed = true;
                    break;
                }
            }
        }
    })
    .await
    .expect("close timeout");
    assert!(closed, "unprovisioned token must close");
    assert!(code_ok, "unprovisioned token must close with 4401");
    wait_for_metric(
        &harness.registry,
        r#"websocket_disconnects_total{reason="unauthorized"} 1"#,
    )
    .await;
}

#[tokio::test]
async fn ws_auth_times_out_if_no_frame() {
    let issuer = start_real_issuer().await;
    let config = WsConfig {
        auth_timeout: Duration::from_millis(150),
        heartbeat_interval: Duration::from_secs(10),
        heartbeat_timeout: Duration::from_secs(30),
        ..WsConfig::default()
    };
    let harness = serve_ws_with_verifier(ws_verifier(&issuer), config).await;
    let mut socket = connect(harness.addr).await;

    let mut closed = false;
    let mut code_ok = false;
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match socket.next().await {
                Some(Ok(ClientWsMessage::Close(frame))) => {
                    closed = true;
                    if let Some(f) = frame
                        && u16::from(f.code) == 4401
                    {
                        code_ok = true;
                    }
                    break;
                }
                Some(Ok(_)) => continue,
                Some(Err(_)) | None => {
                    closed = true;
                    break;
                }
            }
        }
    })
    .await
    .expect("timeout waiting for auth timeout close");
    assert!(closed, "connection must close when no auth frame is sent");
    assert!(code_ok, "auth timeout must close with 4401");
    wait_for_metric(
        &harness.registry,
        r#"websocket_disconnects_total{reason="unauthorized"} 1"#,
    )
    .await;
}

// --- Service principal transport tests (TASK-114) ---

/// Router with shared service principal seams and a supplied room store.
fn service_app(
    principal: Option<Principal>,
    authorizer: Arc<dyn CapabilityResolver>,
    rooms: FakeRoomStore,
    service_store: FakeServicePrincipalStore,
) -> Router {
    let realtime = ws_state(principal.clone());
    let (service_auth, service_principals) = service_seams(service_store);
    api_router(AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(authorizer),
        authentication: Arc::new(StubAuthenticator::new(principal)),
        service_auth,
        service_principals,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: Arc::new(rooms),
        sessions: Arc::new(FakeSessionStore::default()),
        participation: Arc::new(FakeParticipationStore::default()),
        invites: Arc::new(FakeInviteStore::default()),
        code_index: Arc::new(FakeCoordination::default()),
        realtime,
        turn: turn_settings(),
    })
}

fn bearer_request(uri: &str, method: Method, key: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method(method)
        .header(header::AUTHORIZATION, format!("Bearer {key}"))
        .body(Body::empty())
        .expect("request")
}

async fn provisioned_service(
    store: &FakeServicePrincipalStore,
    name: &str,
) -> mahfel_application::service_principals::CreatedServicePrincipal {
    create_service_principal(
        store,
        CreateServicePrincipal {
            name: name.to_owned(),
            display_name: None,
        },
    )
    .await
    .expect("create service principal")
}

#[tokio::test]
async fn service_api_key_authenticates_as_a_service_principal() {
    let service_store = FakeServicePrincipalStore::default();
    let created = provisioned_service(&service_store, "lms-core").await;
    let app = service_app(
        None,
        Arc::new(AllowAll),
        FakeRoomStore::default(),
        service_store.clone(),
    );

    let resp = app
        .oneshot(bearer_request("/v1/auth/me", Method::GET, &created.api_key))
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let expected = created.principal.id.to_string();
    let json = response_json(resp).await;
    assert_eq!(
        json.get("user_id").and_then(|value| value.as_str()),
        Some(expected.as_str()),
        "the principal must be the service actor, not an impersonated user"
    );
}

#[tokio::test]
async fn revoked_service_api_key_is_rejected() {
    let service_store = FakeServicePrincipalStore::default();
    let created = provisioned_service(&service_store, "lms-core").await;
    service_store
        .revoke(created.principal.id)
        .await
        .expect("revoke principal");

    let app = service_app(
        None,
        Arc::new(AllowAll),
        FakeRoomStore::default(),
        service_store,
    );
    let resp = app
        .oneshot(bearer_request("/v1/auth/me", Method::GET, &created.api_key))
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unknown_service_api_key_is_rejected() {
    let app = service_app(
        None,
        Arc::new(AllowAll),
        FakeRoomStore::default(),
        FakeServicePrincipalStore::default(),
    );
    let resp = app
        .oneshot(bearer_request("/v1/auth/me", Method::GET, "mhf_garbage"))
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_service_principals_require_manage_system() {
    let service_store = FakeServicePrincipalStore::default();
    let admin = Principal {
        actor_id: ActorId::new(),
        kind: ActorKind::User,
        display_name: Some("Admin".to_owned()),
        avatar_url: None,
        email: None,
        email_verified: false,
        username: Some("admin".to_owned()),
        roles: vec!["admin".to_owned()],
    };
    let moderator = Principal {
        roles: vec!["moderator".to_owned()],
        ..principal()
    };

    let admin_app = service_app(
        Some(admin),
        Arc::new(ServerCapabilityResolver),
        FakeRoomStore::default(),
        service_store.clone(),
    );
    let resp = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/admin/service-principals")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = send_json(
        &admin_app,
        Method::POST,
        "/v1/admin/service-principals",
        r#"{"name":"lms-core"}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    for (principal, label) in [(principal(), "member"), (moderator, "moderator")] {
        let app = service_app(
            Some(principal),
            Arc::new(ServerCapabilityResolver),
            FakeRoomStore::default(),
            service_store.clone(),
        );
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/v1/admin/service-principals")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "{label} must be denied service principal administration"
        );
    }
}

#[tokio::test]
async fn service_principal_cannot_manage_service_principals() {
    let service_store = FakeServicePrincipalStore::default();
    let created = provisioned_service(&service_store, "lms-core").await;
    let app = service_app(
        None,
        Arc::new(ServerCapabilityResolver),
        FakeRoomStore::default(),
        service_store,
    );
    let resp = app
        .oneshot(bearer_request(
            "/v1/admin/service-principals",
            Method::GET,
            &created.api_key,
        ))
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn service_principal_creates_a_room_and_becomes_manager() {
    let service_store = FakeServicePrincipalStore::default();
    let created = provisioned_service(&service_store, "lms-core").await;
    let rooms = FakeRoomStore::default();
    let app = service_app(None, Arc::new(AllowAll), rooms.clone(), service_store);

    let req = Request::builder()
        .uri("/v1/rooms")
        .method(Method::POST)
        .header(header::AUTHORIZATION, format!("Bearer {}", created.api_key))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"title":"LMS Room"}"#))
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::CREATED);
    let expected_owner = created.principal.id.to_string();
    let json = response_json(resp).await;
    assert_eq!(
        json.get("created_by").and_then(|value| value.as_str()),
        Some(expected_owner.as_str())
    );
    let room_id = RoomId::from_uuid(
        Uuid::parse_str(
            json.get("id")
                .and_then(|value| value.as_str())
                .expect("room id"),
        )
        .expect("valid uuid"),
    );
    let membership = rooms
        .get_member(room_id, created.principal.id)
        .await
        .expect("store")
        .expect("the service principal is a member");
    assert_eq!(membership.role, RoomRole::Manager);
}

/// Like [`serve_ws`], but with the stores seeded so participation frames can
/// resolve a real room, session, and participant.
async fn serve_ws_with(
    principal: Option<Principal>,
    rooms: FakeRoomStore,
    sessions: FakeSessionStore,
    participants: FakeParticipantStore,
) -> WsHarness {
    let registry = prometheus::Registry::new();
    let metrics = WsMetrics::register(&registry).expect("register ws metrics");
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let realtime = WsState::new(
        Arc::new(StubAuthenticator::new(principal.clone())),
        WsConfig::default(),
        metrics,
        Arc::new(GrpcMediaGateway::lazy("http://localhost:50051".to_owned()).expect("media")),
        ParticipationStores {
            participants: Arc::new(participants),
            participation: Arc::new(FakeParticipationStore::default()),
            sessions: Arc::new(sessions),
            rooms: Arc::new(rooms),
        },
        Arc::new(FakeCoordination::default()),
        shutdown_rx.clone(),
    );
    let (service_auth, service_principals) = service_seams(FakeServicePrincipalStore::default());
    let state = AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(AllowAll)),
        authentication: Arc::new(StubAuthenticator::new(principal)),
        service_auth,
        service_principals,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: Arc::new(FakeRoomStore::default()),
        sessions: Arc::new(FakeSessionStore::default()),
        participation: Arc::new(FakeParticipationStore::default()),
        invites: Arc::new(FakeInviteStore::default()),
        code_index: Arc::new(FakeCoordination::default()),
        realtime,
        turn: turn_settings(),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("listener address");
    let task = tokio::spawn(mahfel_telemetry::serve_http(
        listener,
        api_router(state),
        shutdown_rx,
    ));
    WsHarness {
        addr,
        shutdown: shutdown_tx,
        registry,
        _task: task,
    }
}

/// A live session owned by `principal()`, plus the stores that hold it.
fn participation_fixture() -> (
    FakeRoomStore,
    FakeSessionStore,
    FakeParticipantStore,
    Session,
) {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let sessions = FakeSessionStore::default();
    let mut session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    session.start(OffsetDateTime::now_utc()).expect("start");
    sessions.seed(session.clone());
    (rooms, sessions, FakeParticipantStore::default(), session)
}

#[tokio::test]
async fn websocket_join_resolves_a_participant_and_reconnect_reuses_it() {
    let (rooms, sessions, participants, session) = participation_fixture();
    let harness = serve_ws_with(Some(principal()), rooms, sessions, participants).await;

    let mut first = connect(harness.addr).await;
    authenticate(&mut first).await;
    send(
        &mut first,
        &format!(r#"{{"v":1,"type":"join","session_id":"{}"}}"#, session.id),
    )
    .await;
    let joined = next(&mut first).await;
    let joined_text = joined.to_text().expect("text").to_owned();
    assert!(
        joined_text.contains(r#""type":"joined""#),
        "join must be acknowledged: {joined_text}"
    );
    let participant_id = joined_text
        .split(r#""participant_id":""#)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("participant id present")
        .to_owned();

    // A second transport for the same actor is a reconnect, not a new identity.
    let mut second = connect(harness.addr).await;
    authenticate(&mut second).await;
    send(
        &mut second,
        &format!(r#"{{"v":1,"type":"join","session_id":"{}"}}"#, session.id),
    )
    .await;
    let rejoined = next(&mut second).await;
    let rejoined_text = rejoined.to_text().expect("text").to_owned();
    let repeated = rejoined_text
        .split(r#""participant_id":""#)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("participant id present");

    assert_eq!(
        participant_id, repeated,
        "reconnect must reuse the stable participant identity"
    );
}

#[tokio::test]
async fn websocket_join_is_rejected_for_a_non_member() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", ActorId::new(), RoomVisibility::InviteOnly);
    let sessions = FakeSessionStore::default();
    let mut session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    session.start(OffsetDateTime::now_utc()).expect("start");
    sessions.seed(session.clone());

    let harness = serve_ws_with(
        Some(principal()),
        rooms,
        sessions,
        FakeParticipantStore::default(),
    )
    .await;
    let mut socket = connect(harness.addr).await;
    authenticate(&mut socket).await;
    send(
        &mut socket,
        &format!(r#"{{"v":1,"type":"join","session_id":"{}"}}"#, session.id),
    )
    .await;

    let rejected = next(&mut socket).await;
    let text = rejected.to_text().expect("text").to_owned();
    assert!(
        text.contains(r#""type":"join_rejected""#),
        "a non-member must be rejected: {text}"
    );
    assert!(
        text.contains(r#""code":"forbidden""#),
        "the rejection must not reveal room existence: {text}"
    );
}

#[tokio::test]
async fn websocket_leave_is_terminal_and_the_actor_can_rejoin() {
    let (rooms, sessions, participants, session) = participation_fixture();
    let harness = serve_ws_with(Some(principal()), rooms, sessions, participants).await;

    let mut socket = connect(harness.addr).await;
    authenticate(&mut socket).await;
    send(
        &mut socket,
        &format!(r#"{{"v":1,"type":"join","session_id":"{}"}}"#, session.id),
    )
    .await;
    let joined_text = next(&mut socket).await.to_text().expect("text").to_owned();
    let first_id = joined_text
        .split(r#""participant_id":""#)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("participant id")
        .to_owned();

    send(&mut socket, r#"{"v":1,"type":"leave"}"#).await;
    let left_text = next(&mut socket).await.to_text().expect("text").to_owned();
    assert!(
        left_text.contains(r#""type":"left""#),
        "leave must be acknowledged: {left_text}"
    );

    // Rejoining is a new participation, because leaving is terminal.
    send(
        &mut socket,
        &format!(r#"{{"v":1,"type":"join","session_id":"{}"}}"#, session.id),
    )
    .await;
    let rejoined_text = next(&mut socket).await.to_text().expect("text").to_owned();
    let second_id = rejoined_text
        .split(r#""participant_id":""#)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("participant id")
        .to_owned();
    assert_ne!(first_id, second_id, "a rejoin is a new participation");
}

#[tokio::test]
async fn websocket_join_peers_see_the_roster_change() {
    let (rooms, sessions, participants, session) = participation_fixture();
    let harness = serve_ws_with(Some(principal()), rooms, sessions, participants).await;

    let mut owner = connect(harness.addr).await;
    authenticate(&mut owner).await;
    send(
        &mut owner,
        &format!(r#"{{"v":1,"type":"join","session_id":"{}"}}"#, session.id),
    )
    .await;
    let _ = next(&mut owner).await;

    // A second authenticated principal joins the same session; the owner's
    // connection must observe the roster change.
    let mut peer = connect(harness.addr).await;
    authenticate(&mut peer).await;
    send(
        &mut peer,
        &format!(r#"{{"v":1,"type":"join","session_id":"{}"}}"#, session.id),
    )
    .await;
    let _ = next(&mut peer).await;

    let observed = next(&mut owner).await;
    let text = observed.to_text().expect("text").to_owned();
    assert!(
        text.contains(r#""type":"participant_joined""#),
        "an existing member must see the roster change: {text}"
    );
}

// --- Participation record transport tests (TASK-123) ---

/// Router whose participation store the caller controls, so interval fixtures
/// can be seeded directly.
fn participation_app(
    principal: Option<Principal>,
    rooms: FakeRoomStore,
    sessions: FakeSessionStore,
    participation: FakeParticipationStore,
) -> Router {
    let realtime = ws_state(principal.clone());
    let (service_auth, service_principals) = service_seams(FakeServicePrincipalStore::default());
    api_router(AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(ServerCapabilityResolver)),
        authentication: Arc::new(StubAuthenticator::new(principal)),
        service_auth,
        service_principals,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: Arc::new(rooms),
        sessions: Arc::new(sessions),
        participation: Arc::new(participation),
        invites: Arc::new(FakeInviteStore::default()),
        code_index: Arc::new(FakeCoordination::default()),
        realtime,
        turn: turn_settings(),
    })
}

fn interval_at(
    participant_id: ParticipantId,
    session_id: SessionId,
    actor_id: ActorId,
    start_secs: i64,
    end_secs: Option<i64>,
) -> ParticipationInterval {
    let start = OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(start_secs);
    let mut interval = ParticipationInterval::open(participant_id, session_id, actor_id, start);
    if let Some(end) = end_secs {
        let end = OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(end);
        interval.close(end).expect("close fixture interval");
    }
    interval
}

#[tokio::test]
async fn a_manager_reads_session_participation() {
    let rooms = FakeRoomStore::default();
    let manager = principal();
    let room = rooms.seed_room("Algebra", manager.actor_id, RoomVisibility::InviteOnly);
    let sessions = FakeSessionStore::default();
    let mut session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    session.start(OffsetDateTime::now_utc()).expect("start");
    sessions.seed(session.clone());

    let participant = ParticipantId::new();
    let participation = FakeParticipationStore::default();
    participation.seed(interval_at(
        participant,
        session.id,
        manager.actor_id,
        0,
        Some(60),
    ));
    let app = participation_app(Some(manager), rooms, sessions, participation);

    let req = Request::builder()
        .uri(format!("/v1/sessions/{}/participation", session.id))
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    let participants = json
        .get("participants")
        .and_then(|value| value.as_array())
        .expect("participants array");
    assert_eq!(participants.len(), 1);
    assert_eq!(
        participants[0]
            .get("connected_seconds")
            .and_then(|value| value.as_i64()),
        Some(60)
    );
}

#[tokio::test]
async fn a_plain_member_cannot_read_session_participation() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let member = other_principal();
    rooms.add_raw_member(room.id, member.actor_id, RoomRole::Member);
    let sessions = FakeSessionStore::default();
    let session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.seed(session.clone());
    let app = participation_app(
        Some(member),
        rooms,
        sessions,
        FakeParticipationStore::default(),
    );

    let req = Request::builder()
        .uri(format!("/v1/sessions/{}/participation", session.id))
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_participant_reads_its_own_intervals() {
    let rooms = FakeRoomStore::default();
    let actor = principal();
    let room = rooms.seed_room("Algebra", actor.actor_id, RoomVisibility::InviteOnly);
    let sessions = FakeSessionStore::default();
    let session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.seed(session.clone());

    let participant = ParticipantId::new();
    let participation = FakeParticipationStore::default();
    participation.seed(interval_at(
        participant,
        session.id,
        actor.actor_id,
        0,
        Some(30),
    ));
    let app = participation_app(Some(actor), rooms, sessions, participation);

    let req = Request::builder()
        .uri(format!(
            "/v1/sessions/{}/participants/{}/participation",
            session.id, participant
        ))
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    assert_eq!(
        json.get("connected_seconds")
            .and_then(|value| value.as_i64()),
        Some(30)
    );
}

#[tokio::test]
async fn a_manager_corrects_an_interval_and_history_is_retained() {
    let rooms = FakeRoomStore::default();
    let manager = principal();
    let room = rooms.seed_room("Algebra", manager.actor_id, RoomVisibility::InviteOnly);
    let sessions = FakeSessionStore::default();
    let mut session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    session.start(OffsetDateTime::now_utc()).expect("start");
    session.end(OffsetDateTime::now_utc()).expect("end");
    sessions.seed(session.clone());

    let participant = ParticipantId::new();
    let participation = FakeParticipationStore::default();
    let original = interval_at(participant, session.id, manager.actor_id, 0, Some(10));
    let original_id = original.id;
    participation.seed(original);
    let app = participation_app(Some(manager), rooms, sessions, participation);

    let body = serde_json::json!({
        "started_at": "1970-01-01T00:00:00Z",
        "ended_at": "1970-01-01T00:02:00Z",
        "reason": "wrong_end",
    })
    .to_string();
    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/participation/intervals/{original_id}/correct"),
        &body,
    )
    .await;

    let status = resp.status();
    let body_text = response_body_string(resp).await;
    assert_eq!(status, StatusCode::OK, "body: {body_text}");
    let json: serde_json::Value = serde_json::from_str(&body_text).expect("json");
    assert_eq!(
        json.get("connected_seconds")
            .and_then(|value| value.as_i64()),
        Some(120),
        "the replacement carries the corrected duration"
    );

    // The original row survives, marked as superseded.
    let req = Request::builder()
        .uri(format!(
            "/v1/sessions/{}/participants/{}/participation",
            session.id, participant
        ))
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    let json = response_json(resp).await;
    let intervals = json
        .get("intervals")
        .and_then(|value| value.as_array())
        .expect("intervals array");
    assert_eq!(intervals.len(), 2, "history keeps both intervals");
    let superseded = intervals
        .iter()
        .find(|interval| interval.get("superseded_by").is_some())
        .expect("original is retained as superseded");
    assert_eq!(
        superseded.get("correction_reason").and_then(|v| v.as_str()),
        Some("wrong_end")
    );
    assert_eq!(
        json.get("connected_seconds")
            .and_then(|value| value.as_i64()),
        Some(120),
        "a superseded interval must not also count"
    );
}

#[tokio::test]
async fn a_non_manager_cannot_correct_an_interval() {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Algebra", principal().actor_id, RoomVisibility::InviteOnly);
    let member = other_principal();
    rooms.add_raw_member(room.id, member.actor_id, RoomRole::Member);
    let sessions = FakeSessionStore::default();
    let session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.seed(session.clone());

    let participant = ParticipantId::new();
    let participation = FakeParticipationStore::default();
    let original = interval_at(participant, session.id, member.actor_id, 0, Some(10));
    let original_id = original.id;
    participation.seed(original);
    let app = participation_app(Some(member), rooms, sessions, participation);

    let body = serde_json::json!({
        "started_at": "1970-01-01T00:00:00Z",
        "ended_at": "1970-01-01T00:02:00Z",
        "reason": "manual_adjustment",
    })
    .to_string();
    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/participation/intervals/{original_id}/correct"),
        &body,
    )
    .await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// --- Invite link transport tests (TASK-129) ---

/// Router whose invite store the caller controls, so link fixtures and
/// collision retries can be driven directly.
fn invite_app(
    principal: Option<Principal>,
    rooms: FakeRoomStore,
    sessions: FakeSessionStore,
    invites: FakeInviteStore,
) -> Router {
    let realtime = ws_state(principal.clone());
    let (service_auth, service_principals) = service_seams(FakeServicePrincipalStore::default());
    api_router(AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(ServerCapabilityResolver)),
        authentication: Arc::new(StubAuthenticator::new(principal)),
        service_auth,
        service_principals,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: Arc::new(rooms),
        sessions: Arc::new(sessions),
        participation: Arc::new(FakeParticipationStore::default()),
        invites: Arc::new(invites),
        code_index: Arc::new(FakeCoordination::default()),
        realtime,
        turn: turn_settings(),
    })
}

/// A room with `manager` as its manager and `member` as a plain member.
fn invite_fixture(manager: ActorId, member: ActorId) -> (FakeRoomStore, FakeSessionStore, Room) {
    let rooms = FakeRoomStore::default();
    let room = rooms.seed_room("Analytical Engines", manager, RoomVisibility::InviteOnly);
    rooms.add_raw_member(room.id, member, RoomRole::Member);
    let sessions = FakeSessionStore::default();
    (rooms, sessions, room)
}

#[tokio::test]
async fn a_manager_creates_a_room_invite_link() {
    let manager = ActorId::new();
    let member = ActorId::new();
    let (rooms, sessions, room) = invite_fixture(manager, member);
    let app = invite_app(
        Some(principal_for(manager)),
        rooms,
        sessions,
        FakeInviteStore::default(),
    );

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/invite-links", room.id),
        "{}",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = response_json(resp).await;
    let code = json
        .get("code")
        .and_then(|value| value.as_str())
        .expect("code present");
    assert_eq!(code.len(), 10, "the canonical code is ten letters");
    assert!(
        code.chars().all(|c| c.is_ascii_lowercase()),
        "codes are lowercase: {code}"
    );
    assert_eq!(
        json.get("display_code").and_then(|value| value.as_str()),
        Some(format!("{}-{}-{}", &code[0..3], &code[3..7], &code[7..10]).as_str())
    );
    assert_eq!(
        json.get("target_kind").and_then(|value| value.as_str()),
        Some("room")
    );
    assert_eq!(
        json.get("redeemable").and_then(|value| value.as_bool()),
        Some(true)
    );
}

#[tokio::test]
async fn a_manager_creates_a_session_invite_link() {
    let manager = ActorId::new();
    let member = ActorId::new();
    let (rooms, sessions, room) = invite_fixture(manager, member);
    let session = Session::schedule(room.id, None, OffsetDateTime::now_utc());
    sessions.seed(session.clone());
    let app = invite_app(
        Some(principal_for(manager)),
        rooms,
        sessions,
        FakeInviteStore::default(),
    );

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/sessions/{}/invite-links", session.id),
        "{}",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = response_json(resp).await;
    assert_eq!(
        json.get("target_kind").and_then(|value| value.as_str()),
        Some("session")
    );
    assert_eq!(
        json.get("target_id").and_then(|value| value.as_str()),
        Some(session.id.to_string().as_str())
    );
}

#[tokio::test]
async fn a_member_cannot_create_a_link() {
    let manager = ActorId::new();
    let member = ActorId::new();
    let (rooms, sessions, room) = invite_fixture(manager, member);
    let app = invite_app(
        Some(principal_for(member)),
        rooms,
        sessions,
        FakeInviteStore::default(),
    );

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/invite-links", room.id),
        "{}",
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "a plain member must not issue a code"
    );
}

#[tokio::test]
async fn an_outsider_cannot_create_or_list_links() {
    let manager = ActorId::new();
    let member = ActorId::new();
    let (rooms, sessions, room) = invite_fixture(manager, member);
    let outsider = principal_for(ActorId::new());
    let app = invite_app(Some(outsider), rooms, sessions, FakeInviteStore::default());

    // A non-member gets the same 404 as a missing room, so the surface cannot
    // be used to probe which rooms exist.
    let created = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/invite-links", room.id),
        "{}",
    )
    .await;
    assert_eq!(created.status(), StatusCode::NOT_FOUND);

    let listed = send_json(
        &app,
        Method::GET,
        &format!("/v1/rooms/{}/invite-links", room.id),
        "",
    )
    .await;
    assert_eq!(listed.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn link_expiry_is_bounded() {
    let manager = ActorId::new();
    let member = ActorId::new();
    let (rooms, sessions, room) = invite_fixture(manager, member);
    let app = invite_app(
        Some(principal_for(manager)),
        rooms,
        sessions,
        FakeInviteStore::default(),
    );

    for days in [0, -1, mahfel_domain::MAX_EXPIRY_DAYS + 1] {
        let resp = send_json(
            &app,
            Method::POST,
            &format!("/v1/rooms/{}/invite-links", room.id),
            &format!(r#"{{"expiry_days":{days}}}"#),
        )
        .await;
        assert_eq!(
            resp.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "{days} days must be refused"
        );
    }
}

#[tokio::test]
async fn a_manager_lists_and_revokes_a_link() {
    let manager = ActorId::new();
    let member = ActorId::new();
    let (rooms, sessions, room) = invite_fixture(manager, member);
    let invites = FakeInviteStore::default();
    let app = invite_app(
        Some(principal_for(manager)),
        rooms.clone(),
        sessions,
        invites.clone(),
    );

    let created = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/invite-links", room.id),
        "{}",
    )
    .await;
    let json = response_json(created).await;
    let id = json
        .get("id")
        .and_then(|value| value.as_str())
        .expect("link id")
        .to_owned();

    let listed = send_json(
        &app,
        Method::GET,
        &format!("/v1/rooms/{}/invite-links", room.id),
        "",
    )
    .await;
    assert_eq!(listed.status(), StatusCode::OK);
    let json = response_json(listed).await;
    assert_eq!(
        json.get("links")
            .and_then(|value| value.as_array())
            .map(Vec::len),
        Some(1)
    );

    let revoked = send_json(
        &app,
        Method::POST,
        &format!("/v1/invite-links/{id}/revoke"),
        "",
    )
    .await;
    assert_eq!(revoked.status(), StatusCode::OK);
    let json = response_json(revoked).await;
    assert_eq!(
        json.get("status").and_then(|value| value.as_str()),
        Some("revoked")
    );
    assert_eq!(
        json.get("redeemable").and_then(|value| value.as_bool()),
        Some(false),
        "a revoked link is not redeemable"
    );

    // Revocation is terminal.
    let again = send_json(
        &app,
        Method::POST,
        &format!("/v1/invite-links/{id}/revoke"),
        "",
    )
    .await;
    assert_eq!(again.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn rotating_a_link_replaces_the_code_and_retires_the_old_one() {
    let manager = ActorId::new();
    let member = ActorId::new();
    let (rooms, sessions, room) = invite_fixture(manager, member);
    let invites = FakeInviteStore::default();
    let app = invite_app(
        Some(principal_for(manager)),
        rooms.clone(),
        sessions,
        invites.clone(),
    );

    let created = send_json(
        &app,
        Method::POST,
        &format!("/v1/rooms/{}/invite-links", room.id),
        "{}",
    )
    .await;
    let json = response_json(created).await;
    let original_id = json
        .get("id")
        .and_then(|value| value.as_str())
        .expect("link id")
        .to_owned();
    let original_code = json
        .get("code")
        .and_then(|value| value.as_str())
        .expect("code")
        .to_owned();

    let rotated = send_json(
        &app,
        Method::POST,
        &format!("/v1/invite-links/{original_id}/rotate"),
        "{}",
    )
    .await;
    assert_eq!(rotated.status(), StatusCode::OK);
    let json = response_json(rotated).await;
    let replacement_code = json
        .get("code")
        .and_then(|value| value.as_str())
        .expect("code");
    assert_ne!(
        replacement_code, original_code,
        "a rotation must not reuse the code"
    );
    assert_eq!(
        json.get("redeemable").and_then(|value| value.as_bool()),
        Some(true)
    );

    // The retired link is retained and no longer redeemable.
    let all = invites.links();
    assert_eq!(all.len(), 2, "both generations are retained");
    let retired = all
        .iter()
        .find(|link| link.code.as_str() == original_code)
        .expect("retired link retained");
    assert_eq!(retired.status, InviteLinkStatus::Revoked);
    assert!(!retired.is_redeemable_at(OffsetDateTime::now_utc()));
}

#[tokio::test]
async fn a_member_cannot_revoke_a_link_it_does_not_manage() {
    let manager = ActorId::new();
    let member = ActorId::new();
    let (rooms, sessions, room) = invite_fixture(manager, member);
    let invites = FakeInviteStore::default();
    let now = OffsetDateTime::now_utc();
    let link = InviteLink::issue(
        InviteCode::generate(),
        InviteTarget::Room(room.id),
        manager,
        7,
        now,
    )
    .expect("issue");
    invites.seed(link.clone());

    // The member can see the room but does not manage it.
    let app = invite_app(Some(principal_for(member)), rooms, sessions, invites);
    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/invite-links/{}/revoke", link.id),
        "",
    )
    .await;
    // The member can see the room, so the guard reports a denial rather than
    // pretending the link is absent. Link ids are unguessable, so this does not
    // expose one: a caller cannot name a link it has not seen.
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn an_unknown_link_id_is_not_found() {
    let manager = ActorId::new();
    let member = ActorId::new();
    let (rooms, sessions, _room) = invite_fixture(manager, member);
    let app = invite_app(
        Some(principal_for(manager)),
        rooms,
        sessions,
        FakeInviteStore::default(),
    );

    let resp = send_json(
        &app,
        Method::POST,
        &format!("/v1/invite-links/{}/revoke", Uuid::new_v4()),
        "",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
