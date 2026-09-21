//! Authorization seam.
//!
//! Identity is modelled separately from policy: [`Principal`] is an
//! authenticated actor and [`Authorizer`] decides whether that actor may
//! perform an [`Action`]. Policy is deny-by-default: a transport must not
//! attach a principal without authentication, and an [`Authorizer`] must
//! return an error unless an explicit rule grants access.

use crate::error::{ApplicationError, ErrorKind};
use mahfel_domain::{ActorId, ActorKind};
use thiserror::Error;

/// An authenticated actor.
///
/// The actor may be a user or a service principal. `kind` is carried from the
/// verified credential so policy can reason about a machine actor without a
/// database lookup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Principal {
    pub actor_id: ActorId,
    pub kind: ActorKind,
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
    /// System-wide administration (e.g., manage realm, view admin panel).
    ManageSystem,
    /// Classroom moderation (e.g., mute, remove members).
    ModerateRoom,
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

/// Role-based authorizer for global roles.
///
/// Global roles (`admin`, `moderator`) are attached to Keycloak-issued user
/// tokens; all other actors are `member` (explicit or implicit) and are handled
/// explicitly within the app via `room_memberships`. Service principals carry
/// no global roles, so they may create and own rooms and hold room
/// memberships, but they can never `ManageSystem` or globally `ModerateRoom`
/// through this policy. Deny-by-default remains: an action is allowed only by
/// the explicit rules below.
#[derive(Clone, Copy, Debug, Default)]
pub struct RoleBasedAuthorizer;

impl RoleBasedAuthorizer {
    fn has_role(principal: &Principal, role: &str) -> bool {
        principal.roles.iter().any(|r| r.eq_ignore_ascii_case(role))
    }

    fn is_admin(principal: &Principal) -> bool {
        Self::has_role(principal, "admin")
    }

    fn is_moderator(principal: &Principal) -> bool {
        Self::has_role(principal, "moderator")
    }
}

impl Authorizer for RoleBasedAuthorizer {
    fn authorize(&self, principal: &Principal, action: Action) -> Result<(), AuthorizationError> {
        match action {
            Action::ViewOwnIdentity | Action::CreateRoom => Ok(()),
            Action::ManageSystem => {
                if Self::is_admin(principal) {
                    Ok(())
                } else {
                    Err(AuthorizationError::Denied)
                }
            }
            Action::ModerateRoom => {
                if Self::is_admin(principal) || Self::is_moderator(principal) {
                    Ok(())
                } else {
                    Err(AuthorizationError::Denied)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahfel_domain::{ActorId, ActorKind};
    use uuid::Uuid;

    fn principal_with_roles(roles: &[&str]) -> Principal {
        Principal {
            actor_id: ActorId::from_uuid(Uuid::nil()),
            kind: ActorKind::User,
            display_name: None,
            avatar_url: None,
            email: None,
            email_verified: false,
            username: None,
            roles: roles.iter().map(|r| (*r).to_owned()).collect(),
        }
    }

    fn service_principal_with_roles(roles: &[&str]) -> Principal {
        Principal {
            kind: ActorKind::Service,
            ..principal_with_roles(roles)
        }
    }

    #[test]
    fn admin_can_manage_system_and_moderate() {
        let admin = principal_with_roles(&["admin"]);
        assert!(
            RoleBasedAuthorizer
                .authorize(&admin, Action::ManageSystem)
                .is_ok()
        );
        assert!(
            RoleBasedAuthorizer
                .authorize(&admin, Action::ModerateRoom)
                .is_ok()
        );
    }

    #[test]
    fn moderator_can_moderate_but_not_manage_system() {
        let moderator = principal_with_roles(&["moderator"]);
        assert!(
            RoleBasedAuthorizer
                .authorize(&moderator, Action::ModerateRoom)
                .is_ok()
        );
        assert!(
            RoleBasedAuthorizer
                .authorize(&moderator, Action::ManageSystem)
                .is_err()
        );
    }

    #[test]
    fn member_can_only_view_and_create() {
        let member = principal_with_roles(&["member"]);
        let implicit = principal_with_roles(&[]);
        for principal in [member, implicit] {
            assert!(
                RoleBasedAuthorizer
                    .authorize(&principal, Action::ViewOwnIdentity)
                    .is_ok()
            );
            assert!(
                RoleBasedAuthorizer
                    .authorize(&principal, Action::CreateRoom)
                    .is_ok()
            );
            assert!(
                RoleBasedAuthorizer
                    .authorize(&principal, Action::ManageSystem)
                    .is_err()
            );
            assert!(
                RoleBasedAuthorizer
                    .authorize(&principal, Action::ModerateRoom)
                    .is_err()
            );
        }
    }

    #[test]
    fn service_principal_can_create_but_cannot_manage_system_or_moderate() {
        let service = service_principal_with_roles(&[]);
        assert!(
            RoleBasedAuthorizer
                .authorize(&service, Action::ViewOwnIdentity)
                .is_ok()
        );
        assert!(
            RoleBasedAuthorizer
                .authorize(&service, Action::CreateRoom)
                .is_ok()
        );
        assert!(
            RoleBasedAuthorizer
                .authorize(&service, Action::ManageSystem)
                .is_err()
        );
        assert!(
            RoleBasedAuthorizer
                .authorize(&service, Action::ModerateRoom)
                .is_err()
        );
    }

    #[test]
    fn deny_by_default_for_unknown_action_roles() {
        let member = principal_with_roles(&["member"]);
        assert!(
            RoleBasedAuthorizer
                .authorize(&member, Action::ModerateRoom)
                .is_err()
        );
    }
}
