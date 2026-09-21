//! Authentication (bearer token verification) and authorization policy.
//!
//! `mahfel-api` is a resource server (ADR 0019): it verifies the Keycloak
//! access token on every request and builds the principal from the
//! `app_user_id` claim that provisioning links into the token. Normal requests
//! never touch the identity database; only `/v1/auth/provision` writes it.
//!
//! The module is split by responsibility: token verification, the claims to
//! principal mapping, the axum extractors, and the authorization policy.

mod extract;
mod policy;
mod principal;
mod service_key;
mod verification;

pub use extract::{Authenticated, VerifiedToken};
pub use policy::{AuthState, authorize_global, capabilities_in_room};
pub use principal::principal_from_claims;
pub use service_key::{ServiceKeyAuthenticator, StoreServiceKeyAuthenticator};
pub use verification::{AuthenticationError, RequestAuthenticator, TokenAuthentication};
