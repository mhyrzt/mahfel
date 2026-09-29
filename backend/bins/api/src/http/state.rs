use crate::http::{
    auth::{AuthState, RequestAuthenticator, ServiceKeyAuthenticator},
    service_principals::ServicePrincipalAdmin,
    source_address::SourceAddressResolver,
    ws::WsState,
};
use mahfel_application::{
    ports::{
        AdmissionStore, ChatStore, CollaborationStore, CoordinationStore, FileStorage, FileStore,
        InviteStore, ParticipationStore, RoomStore, SessionStore, UserAttributeWriter, UserStore,
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
    pub users: Arc<dyn UserStore>,
    pub chat: Arc<dyn ChatStore>,
    pub recordings: Option<Arc<dyn mahfel_application::ports::RecordingStore>>,
    pub collaboration: Option<Arc<dyn CollaborationStore>>,
    pub files: Arc<dyn FileStore>,
    pub storage: Arc<dyn FileStorage>,
    pub sessions: Arc<dyn SessionStore>,
    pub participation: Arc<dyn ParticipationStore>,
    pub admissions: Arc<dyn AdmissionStore>,
    pub invites: Arc<dyn InviteStore>,
    /// Ephemeral counters behind invite-code rate limiting. Redis, because the
    /// window must be shared across replicas and is replaceable state.
    pub code_index: Arc<dyn CoordinationStore>,
    /// Attributes a redemption attempt to a source address.
    pub source_address: Arc<SourceAddressResolver>,
    pub realtime: WsState,
    pub turn: TurnSettings,
}
