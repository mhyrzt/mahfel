use crate::http::{
    auth::{AuthState, RequestAuthenticator, ServiceKeyAuthenticator},
    service_principals::ServicePrincipalAdmin,
    ws::WsState,
};
use mahfel_application::{
    ports::{RoomStore, UserAttributeWriter},
    users::IdentityProvisioner,
};
use mahfel_config::TurnSettings;
use std::sync::Arc;

/// Store bound required by the API transport.
///
/// [`RoomStore`] uses return-position `impl Future`, so it is not
/// dyn-compatible; the router state is generic over the concrete adapter
/// instead of holding a boxed trait object.
pub trait RoomStoreState: RoomStore + Clone + Send + Sync + 'static {}

impl<T> RoomStoreState for T where T: RoomStore + Clone + Send + Sync + 'static {}

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
    pub realtime: WsState,
    pub turn: TurnSettings,
}
