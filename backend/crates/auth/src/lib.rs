//! Provider-neutral OIDC identity normalization and Keycloak access-token
//! verification (ADR 0019).
//!
//! The crate maps provider-specific claims into [`AuthenticatedIdentity`] at
//! the authentication boundary. Authorization and persistence consume only the
//! normalized type, so vendor logic does not leak into handlers.

mod claims;
mod jwks;
mod verify;

pub use claims::{
    AuthenticatedIdentity, ClaimsError, DEFAULT_LEEWAY, DiscoveryDocument, PrincipalKind,
    TokenError, normalize_claims,
};
pub use jwks::{JwksCache, KeySet, SharedJwks, VerificationKey};
pub use verify::{TokenVerifier, TokenVerifierConfig, unix_now};
