pub mod admission_store;
pub mod authorization;
pub mod block_store;
pub mod coordination;
pub mod event_publisher;
pub mod futures;
pub mod identity_store;
pub mod invite_store;
pub mod media_gateway;
pub mod media_node_registry;
pub mod membership;
pub mod participant_store;
pub mod participation_store;
pub mod room_store;
pub mod service_principal_store;
pub mod session_store;
pub mod user_attribute;
pub mod user_store;

pub use admission_store::AdmissionStore;
pub use authorization::{
    AuthorizationError, Capability, CapabilityContext, CapabilityResolver, CapabilitySet,
    Principal, require_capability, resolve_capabilities,
};
pub use block_store::BlockStore;
pub use coordination::{CoordinationError, CoordinationStore};
pub use event_publisher::{DomainEvent, EventPublisher, PublishError};
pub use futures::BoxFuture;
pub use identity_store::{IdentityProfile, IdentityStore, ProvisionedIdentity};
pub use invite_store::InviteStore;
pub use media_gateway::{MediaGateway, MediaGatewayError};
pub use media_node_registry::{MediaNodeRecord, MediaNodeRegistry};
pub use membership::{Membership, MembershipStore};
pub use participant_store::ParticipantStore;
pub use participation_store::{ParticipantDuration, ParticipationStore};
pub use room_store::RoomStore;
pub use service_principal_store::ServicePrincipalStore;
pub use session_store::SessionStore;
pub use user_attribute::{AttributeError, UserAttributeWriter};
pub use user_store::UserStore;

use thiserror::Error;

/// Generic storage failure for application ports.
///
/// Infrastructure adapters map their concrete errors (`sqlx::Error`,
/// `DatabaseError`, etc.) into this type so `mahfel-application`
/// never depends on `sqlx`/`lapin`/`tonic` directly.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("storage failure: {0}")]
    Unexpected(String),
    #[error("entity already exists: {0}")]
    Conflict(String),
    #[error("entity not found: {0}")]
    NotFound(String),
    /// A referenced row is missing (for example a foreign key target), so the
    /// operation cannot succeed. Distinct from [`StoreError::NotFound`], which
    /// means the requested entity itself was absent.
    #[error("referenced entity not found: {0}")]
    MissingReference(String),
}

impl StoreError {
    pub fn unexpected<E: std::fmt::Display>(err: E) -> Self {
        Self::Unexpected(err.to_string())
    }
}
