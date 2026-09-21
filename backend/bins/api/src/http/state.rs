use crate::http::{
    auth::{AuthState, RequestAuthenticator, ServiceKeyAuthenticator},
    service_principals::ServicePrincipalAdmin,
    ws::WsState,
};
use mahfel_application::{
    ports::{RoomStore, SessionStore, UserAttributeWriter},
    users::IdentityProvisioner,
};
use mahfel_config::TurnSettings;
use std::sync::Arc;

/// Shared router state for the product API.
///
/// Every store port returns a boxed future, so it is dyn-compatible: the state
/// holds one `Arc<dyn Port>` per capability instead of a generic parameter per
/// store. Adding a store is a new field, not a new type parameter threaded
/// through every route signature.
#[derive(Clone)]
pub struct AppState {
    pub service: &'static str,
    pub version: &'static str,
    pub node_id: String,
    pub auth: AuthState,
    pub authentication: Arc<dyn RequestAuthenticator>,
    pub service_auth: Arc<dyn ServiceKeyAuthenticator>,
    pub service_principals: Arc<dyn ServicePrincipalAdmin>,
    pub provisioner: Arc<dyn IdentityProvisioner>,
    pub linker: Arc<dyn UserAttributeWriter>,
    pub rooms: Arc<dyn RoomStore>,
    pub sessions: Arc<dyn SessionStore>,
    pub realtime: WsState,
    pub turn: TurnSettings,
}
