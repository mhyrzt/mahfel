use crate::http::{
    auth::{AuthState, RequestAuthenticator, ServiceKeyAuthenticator},
    service_principals::ServicePrincipalAdmin,
    ws::WsState,
};
use mahfel_application::{
    ports::{RoomStore, SessionStore, StoreError, UserAttributeWriter},
    users::IdentityProvisioner,
};
use mahfel_config::TurnSettings;
use mahfel_domain::{RoomId, Session, SessionId};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Store bound required by the API transport.
///
/// [`RoomStore`] uses return-position `impl Future`, so it is not
/// dyn-compatible; the router state is generic over the concrete adapter
/// instead of holding a boxed trait object.
pub trait RoomStoreState: RoomStore + Clone + Send + Sync + 'static {}

impl<T> RoomStoreState for T where T: RoomStore + Clone + Send + Sync + 'static {}

/// Object-safe seam over [`SessionStore`].
///
/// [`SessionStore`] returns `impl Future`, so it cannot be stored as a trait
/// object. Sessions are used by only a few routes, so the router keeps them
/// behind this small seam instead of threading a second generic parameter
/// through every route signature. [`SessionStoreRef`] adapts it back to the
/// application port for the session use cases.
pub trait SessionStoreHandle: Send + Sync + 'static {
    fn get<'a>(
        &'a self,
        id: SessionId,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Session>, StoreError>> + Send + 'a>>;

    fn insert<'a>(
        &'a self,
        session: &'a Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>>;

    fn save<'a>(
        &'a self,
        session: &'a Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>>;

    fn list_for_room<'a>(
        &'a self,
        room_id: RoomId,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, StoreError>> + Send + 'a>>;
}

impl<T> SessionStoreHandle for T
where
    T: SessionStore + 'static,
{
    fn get<'a>(
        &'a self,
        id: SessionId,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Session>, StoreError>> + Send + 'a>> {
        Box::pin(SessionStore::get(self, id))
    }

    fn insert<'a>(
        &'a self,
        session: &'a Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(SessionStore::insert(self, session))
    }

    fn save<'a>(
        &'a self,
        session: &'a Session,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(SessionStore::save(self, session))
    }

    fn list_for_room<'a>(
        &'a self,
        room_id: RoomId,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Session>, StoreError>> + Send + 'a>> {
        Box::pin(SessionStore::list_for_room(self, room_id))
    }
}

/// [`SessionStore`] adapter over the object-safe [`SessionStoreHandle`] seam.
#[derive(Clone)]
pub struct SessionStoreRef(Arc<dyn SessionStoreHandle>);

impl SessionStoreRef {
    #[must_use]
    pub fn new(store: impl SessionStore + 'static) -> Self {
        Self(Arc::new(store))
    }
}

impl SessionStore for SessionStoreRef {
    async fn get(&self, id: SessionId) -> Result<Option<Session>, StoreError> {
        self.0.get(id).await
    }

    async fn insert(&self, session: &Session) -> Result<(), StoreError> {
        self.0.insert(session).await
    }

    async fn save(&self, session: &Session) -> Result<(), StoreError> {
        self.0.save(session).await
    }

    async fn list_for_room(&self, room_id: RoomId) -> Result<Vec<Session>, StoreError> {
        self.0.list_for_room(room_id).await
    }
}

/// Shared router state for the product API.
#[derive(Clone)]
pub struct AppState<R> {
    pub service: &'static str,
    pub version: &'static str,
    pub node_id: String,
    pub auth: AuthState,
    pub authentication: Arc<dyn RequestAuthenticator>,
    pub service_auth: Arc<dyn ServiceKeyAuthenticator>,
    pub service_principals: Arc<dyn ServicePrincipalAdmin>,
    pub provisioner: Arc<dyn IdentityProvisioner>,
    pub linker: Arc<dyn UserAttributeWriter>,
    pub rooms: R,
    pub sessions: SessionStoreRef,
    pub realtime: WsState,
    pub turn: TurnSettings,
}
