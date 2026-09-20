use super::{
    AppState, api_router,
    auth::{AuthState, AuthenticationError, RequestAuthenticator, TokenAuthentication},
    cors_layer,
    ws::{WsConfig, WsMetrics, WsState},
};
use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Method, Request, StatusCode, header},
};
use futures_util::{SinkExt, StreamExt, future::BoxFuture};
use mahfel_application::ports::{
    Action, AllowAuthenticated, AttributeError, AuthorizationError, Authorizer, Principal,
    ProvisionedIdentity, RoomStore, StoreError, UserAttributeWriter,
};
use mahfel_application::users::{IdentityProvisioner, SyncIdentity, SyncIdentityError};
use mahfel_auth::AuthenticatedIdentity;
use mahfel_auth::{TokenVerifier, TokenVerifierConfig};
use mahfel_domain::{
    DisplayName, Identity, Issuer, Room, RoomId, RoomMember, RoomRole, RoomTitle, RoomVisibility,
    Subject, User, UserId, UserStatus,
};
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
    fn seed_room(&self, title: &str, owner: UserId, visibility: RoomVisibility) -> Room {
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

    fn add_raw_member(&self, room_id: RoomId, user_id: UserId, role: RoomRole) {
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
    async fn get(&self, id: RoomId) -> Result<Option<Room>, StoreError> {
        Ok(self
            .inner
            .lock()
            .expect("fake store lock")
            .rooms
            .iter()
            .find(|room| room.id == id)
            .cloned())
    }

    async fn insert(&self, room: &Room) -> Result<(), StoreError> {
        self.inner
            .lock()
            .expect("fake store lock")
            .rooms
            .push(room.clone());
        Ok(())
    }

    async fn save(&self, room: &Room) -> Result<(), StoreError> {
        let mut inner = self.inner.lock().expect("fake store lock");
        let slot = inner
            .rooms
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
        let mut inner = self.inner.lock().expect("fake store lock");
        inner.rooms.push(room.clone());
        inner.members.push(manager.clone());
        Ok(())
    }

    async fn list_for_member(
        &self,
        user_id: UserId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Room>, StoreError> {
        let inner = self.inner.lock().expect("fake store lock");
        let mut rooms: Vec<Room> = inner
            .rooms
            .iter()
            .filter(|room| {
                inner
                    .members
                    .iter()
                    .any(|member| member.room_id == room.id && member.user_id == user_id)
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
    }

    async fn get_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<Option<RoomMember>, StoreError> {
        Ok(self
            .inner
            .lock()
            .expect("fake store lock")
            .members
            .iter()
            .find(|member| member.room_id == room_id && member.user_id == user_id)
            .cloned())
    }

    async fn list_members(&self, room_id: RoomId) -> Result<Vec<RoomMember>, StoreError> {
        Ok(self
            .inner
            .lock()
            .expect("fake store lock")
            .members
            .iter()
            .filter(|member| member.room_id == room_id)
            .cloned()
            .collect())
    }

    async fn add_member(&self, member: &RoomMember) -> Result<(), StoreError> {
        let mut inner = self.inner.lock().expect("fake store lock");
        if inner.members.iter().any(|existing| {
            existing.room_id == member.room_id && existing.user_id == member.user_id
        }) {
            return Err(StoreError::Conflict("already a member".to_owned()));
        }
        inner.members.push(member.clone());
        Ok(())
    }

    async fn remove_member(&self, room_id: RoomId, user_id: UserId) -> Result<bool, StoreError> {
        let mut inner = self.inner.lock().expect("fake store lock");
        let managers = inner
            .members
            .iter()
            .filter(|member| member.room_id == room_id && member.role == RoomRole::Manager)
            .count();
        let before = inner.members.len();
        inner.members.retain(|member| {
            let is_target = member.room_id == room_id && member.user_id == user_id;
            !is_target || (member.role == RoomRole::Manager && managers <= 1)
        });
        Ok(inner.members.len() < before)
    }
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
                subject: principal.user_id.to_string(),
                issuer: "https://auth.test/realms/rastai".to_owned(),
                app_user_id: self.provisioned.then(|| principal.user_id.to_string()),
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
            .map(UserId::from_uuid)
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

fn ws_state(principal: Option<Principal>) -> WsState {
    let (_shutdown_tx, shutdown_rx) = watch::channel(false);
    let metrics = WsMetrics::register(&prometheus::Registry::new()).expect("register ws metrics");
    WsState::new(
        Arc::new(StubAuthenticator::new(principal)),
        WsConfig::default(),
        metrics,
        shutdown_rx,
    )
}

fn state(principal: Option<Principal>, authorizer: Arc<dyn Authorizer>) -> AppState<FakeRoomStore> {
    state_with(principal, authorizer, true)
}

fn state_unprovisioned(
    principal: Option<Principal>,
    authorizer: Arc<dyn Authorizer>,
) -> AppState<FakeRoomStore> {
    state_with(principal, authorizer, false)
}

fn state_with(
    principal: Option<Principal>,
    authorizer: Arc<dyn Authorizer>,
    provisioned: bool,
) -> AppState<FakeRoomStore> {
    let realtime = ws_state(principal.clone());
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
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: FakeRoomStore::default(),
        realtime,
    }
}

fn principal_with(value: &str) -> Principal {
    Principal {
        user_id: UserId::from_uuid(Uuid::parse_str(value).expect("valid uuid")),
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

fn app() -> Router {
    api_router(state(None, Arc::new(AllowAuthenticated)))
}

fn authenticated_app() -> Router {
    api_router(state(Some(principal()), Arc::new(AllowAuthenticated)))
}

fn authenticated_room_app(principal: Principal, store: FakeRoomStore) -> Router {
    let realtime = ws_state(Some(principal.clone()));
    api_router(AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(AllowAuthenticated)),
        authentication: Arc::new(StubAuthenticator::new(Some(principal))),
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: store,
        realtime,
    })
}

struct DenyAll;

impl Authorizer for DenyAll {
    fn authorize(&self, _principal: &Principal, _action: Action) -> Result<(), AuthorizationError> {
        Err(AuthorizationError::Denied)
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
    let app = api_router(state_unprovisioned(
        Some(principal()),
        Arc::new(AllowAuthenticated),
    ));
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
    let app = api_router(state_unprovisioned(
        Some(principal()),
        Arc::new(AllowAuthenticated),
    ));
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
    use mahfel_application::ports::RoleBasedAuthorizer;

    let admin = Principal {
        user_id: principal().user_id,
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

    let admin_app = api_router(state(Some(admin), Arc::new(RoleBasedAuthorizer)));
    let req = Request::builder()
        .uri("/v1/admin/ping")
        .body(Body::empty())
        .expect("request");
    let resp = admin_app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);

    for (principal, label) in [(moderator, "moderator"), (member, "member")] {
        let app = api_router(state(Some(principal), Arc::new(RoleBasedAuthorizer)));
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
    let app = api_router(state(None, Arc::new(AllowAuthenticated))).layer(cors_layer(origin));

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
    let room = store.seed_room("Algebra", principal().user_id, RoomVisibility::Public);
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
    let room = store.seed_room("Algebra", principal().user_id, RoomVisibility::Public);
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
    store.seed_room("Owned", principal().user_id, RoomVisibility::InviteOnly);
    store.seed_room("Other", other_principal().user_id, RoomVisibility::Public);
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
    let room = store.seed_room("Open", other_principal().user_id, RoomVisibility::Public);
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
            .get_member(room.id, principal().user_id)
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
        other_principal().user_id,
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
    let room = store.seed_room("Algebra", principal().user_id, RoomVisibility::InviteOnly);
    let target = other_principal().user_id;
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
    let room = store.seed_room("Algebra", principal().user_id, RoomVisibility::InviteOnly);
    store.add_raw_member(room.id, other_principal().user_id, RoomRole::Member);
    let app = authenticated_room_app(other_principal(), store);
    let third = principal_with("00000000-0000-0000-0000-000000000011").user_id;

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
    let room = store.seed_room("Algebra", principal().user_id, RoomVisibility::InviteOnly);
    let app = authenticated_room_app(other_principal(), store);
    let target = principal_with("00000000-0000-0000-0000-000000000011").user_id;

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
    let room = store.seed_room("Algebra", principal().user_id, RoomVisibility::InviteOnly);
    let owner = principal().user_id;
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
    let room = store.seed_room("Algebra", principal().user_id, RoomVisibility::InviteOnly);
    let member = other_principal().user_id;
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
    let room = store.seed_room("Algebra", principal().user_id, RoomVisibility::InviteOnly);
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
    let room = store.seed_room("Algebra", principal().user_id, RoomVisibility::InviteOnly);
    store.add_raw_member(room.id, other_principal().user_id, RoomRole::Member);
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
    let room = store.seed_room("Open", other_principal().user_id, RoomVisibility::Public);

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
        shutdown_rx.clone(),
    );
    let state = AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(AllowAuthenticated)),
        authentication: Arc::new(StubAuthenticator::new(principal)),
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: FakeRoomStore::default(),
        realtime,
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
    let realtime = WsState::new(Arc::clone(&auth), config, metrics, shutdown_rx.clone());
    let state = AppState {
        service: "mahfel-api-test",
        version: "0.1.0",
        node_id: "node-test".to_owned(),
        auth: AuthState::new(Arc::new(AllowAuthenticated)),
        authentication: auth,
        provisioner: Arc::new(StubProvisioner),
        linker: Arc::new(StubLinker),
        rooms: FakeRoomStore::default(),
        realtime,
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
