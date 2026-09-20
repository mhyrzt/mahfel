//! Authorization seam.
//!
//! Identity is modelled separately from policy: [`Principal`] is an
//! authenticated actor and [`Authorizer`] decides whether that actor may
//! perform an [`Action`]. Policy is deny-by-default: a transport must not
//! attach a principal without authentication, and an [`Authorizer`] must
//! return an error unless an explicit rule grants access.

use crate::error::{ApplicationError, ErrorKind};
use mahfel_domain::UserId;
use thiserror::Error;

/// An authenticated actor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Principal {
    pub user_id: UserId,
    /// Local display name resolved at provisioning time.
    pub display_name: Option<String>,
    /// Resolved avatar link (user-owned first, provider claim second).
    pub avatar_url: Option<String>,
    /// Provider-verified profile fields, for responses and authorization
    /// context. Never populated from unverified input.
    pub email: Option<String>,
    pub email_verified: bool,
    pub username: Option<String>,
    pub roles: Vec<String>,
}

/// An operation a principal may attempt.
///
/// Actions are added as use cases appear; the interim policy below does not
/// yet distinguish between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    ViewOwnIdentity,
    CreateRoom,
}

/// Why an authorization attempt failed.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum AuthorizationError {
    #[error("authentication required")]
    Unauthenticated,
    #[error("permission denied")]
    Denied,
}

impl ApplicationError for AuthorizationError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Unauthenticated => ErrorKind::Unauthorized,
            Self::Denied => ErrorKind::Forbidden,
        }
    }
}

/// Policy interface. Implementations must deny unless a rule grants access.
pub trait Authorizer: Send + Sync {
    fn authorize(&self, principal: &Principal, action: Action) -> Result<(), AuthorizationError>;
}

/// Interim policy: every authenticated principal may perform every action.
///
/// This exists only until the classroom role model (M2-07) replaces it. It is
/// intentionally permissive but never bypasses authentication, because a
/// [`Principal`] can only be produced by an authenticated transport.
#[derive(Clone, Copy, Debug, Default)]
pub struct AllowAuthenticated;

impl Authorizer for AllowAuthenticated {
    fn authorize(&self, _principal: &Principal, _action: Action) -> Result<(), AuthorizationError> {
        Ok(())
    }
}
