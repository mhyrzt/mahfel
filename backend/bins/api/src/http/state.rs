use crate::http::{
    auth::{AuthState, RequestAuthenticator, ServiceKeyAuthenticator},
    service_principals::ServicePrincipalAdmin,
    ws::WsState,
};
use mahfel_application::{
    ports::{
        CoordinationStore, InviteStore, ParticipationStore, RoomStore, SessionStore,
        UserAttributeWriter,
    },
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
    pub participation: Arc<dyn ParticipationStore>,
    pub invites: Arc<dyn InviteStore>,
    /// Ephemeral counters behind invite-code rate limiting. Redis, because the
    /// window must be shared across replicas and is replaceable state.
    ///
    /// Wired now and consumed by the redemption flow (M4-14), which is the only
    /// caller of the limiter; nothing on this surface reads it yet.
    #[expect(
        dead_code,
        reason = "held for the M4-14 redemption flow, which is the limiter's first caller"
    )]
    pub code_index: Arc<dyn CoordinationStore>,
    pub realtime: WsState,
    pub turn: TurnSettings,
}
