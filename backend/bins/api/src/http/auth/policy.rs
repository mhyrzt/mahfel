use crate::http::error::{ApiError, api_error_from};
use mahfel_application::{
    policy::resolve_room_capabilities,
    ports::{
        AuthorizationError, Capability, CapabilityContext, CapabilityResolver, CapabilitySet,
        MembershipStore, Principal,
    },
};
use mahfel_domain::RoomId;
use std::sync::Arc;

/// Authorization policy for the API.
///
/// Authentication happens per request from the bearer token; this type holds
/// the [`CapabilityResolver`] and the checks built on it. REST and WebSocket
/// both go through here, so a control hidden in the UI and a request forged by
/// hand are decided identically.
#[derive(Clone)]
pub struct AuthState {
    resolver: Arc<dyn CapabilityResolver>,
}

impl AuthState {
    #[must_use]
    pub fn new(resolver: Arc<dyn CapabilityResolver>) -> Self {
        Self { resolver }
    }

    /// Capabilities the principal holds in a context. Empty means deny.
    #[must_use]
    pub fn capabilities(&self, principal: &Principal, context: CapabilityContext) -> CapabilitySet {
        self.resolver.resolve(principal, context)
    }

    /// Fail unless the principal holds `capability` in `context`.
    pub fn require(
        &self,
        principal: &Principal,
        context: CapabilityContext,
        capability: Capability,
    ) -> Result<(), AuthorizationError> {
        if self.capabilities(principal, context).has(capability) {
            return Ok(());
        }
        Err(AuthorizationError::Denied)
    }
}

api_error_from!(AuthorizationError);

/// Run a global capability check, mapping denials into the HTTP error envelope.
///
/// Global capabilities (`ManageSystem`, `CreateRoom`, `ViewOwnIdentity`) depend
/// only on verified roles, so they take no room context.
pub fn authorize_global(
    state: &crate::http::state::AppState,
    principal: &Principal,
    capability: Capability,
) -> Result<(), ApiError> {
    state
        .auth
        .require(principal, CapabilityContext::global(), capability)?;
    Ok(())
}

/// Capabilities the caller holds in one room, for introspection.
pub async fn capabilities_in_room<S>(
    memberships: &S,
    principal: &Principal,
    room_id: RoomId,
) -> Result<CapabilitySet, ApiError>
where
    S: MembershipStore + ?Sized,
{
    Ok(resolve_room_capabilities(memberships, principal, room_id).await?)
}
