//! Authorization seam.
//!
//! Identity is modelled separately from policy: [`Principal`] is an
//! authenticated actor and [`CapabilityResolver`] decides which
//! [`Capability`] values that actor holds in a given context. Policy is
//! deny-by-default: a transport must not attach a principal without
//! authentication, and a resolver must return an empty set unless an explicit
//! rule grants a capability.
//!
//! Capabilities replace a fixed role enum. Global roles, per-room membership,
//! and the service-principal distinction all feed one resolution step, so a
//! vertical maps its vocabulary onto capabilities instead of adding roles
//! (ADR 0020).

use crate::error::{ApplicationError, ErrorKind};
use mahfel_domain::{ActorId, ActorKind, RoomRole};
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

/// An operation a principal may perform.
///
/// Capabilities are either global (they depend only on verified global roles)
/// or room-scoped (they also depend on the caller's membership in one room).
/// [`CapabilityContext`] carries exactly the input each kind needs, so no
/// capability silently ignores context it was given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    /// System-wide administration: realms, service principals, admin panel.
    ManageSystem,
    /// Create a room. Any authenticated actor may; ownership is a separate fact.
    CreateRoom,
    /// Read one's own identity. Never gated on room context.
    ViewOwnIdentity,
    /// Enter a session in a room the caller may see.
    Join,
    /// Send audio or video to the session.
    PublishMedia,
    /// Share a screen.
    ShareScreen,
    /// Mute another participant's published media.
    MuteOthers,
    /// Remove another participant from the session.
    RemoveParticipant,
    /// Admit or reject a caller waiting in the lobby.
    AdmitParticipant,
    /// Start, end, cancel, or schedule a session, and change its settings.
    ManageSession,
}

impl Capability {
    /// Whether this capability depends on room membership.
    ///
    /// Global capabilities are decided from verified roles alone; room-scoped
    /// capabilities require the caller's membership in the room the operation
    /// targets. Documented per capability so the split is not inferred.
    #[must_use]
    pub const fn is_room_scoped(self) -> bool {
        matches!(
            self,
            Self::Join
                | Self::PublishMedia
                | Self::ShareScreen
                | Self::MuteOthers
                | Self::RemoveParticipant
                | Self::AdmitParticipant
                | Self::ManageSession
        )
    }

    /// Stable name for documentation, telemetry, and API responses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ManageSystem => "manage_system",
            Self::CreateRoom => "create_room",
            Self::ViewOwnIdentity => "view_own_identity",
            Self::Join => "join",
            Self::PublishMedia => "publish_media",
            Self::ShareScreen => "share_screen",
            Self::MuteOthers => "mute_others",
            Self::RemoveParticipant => "remove_participant",
            Self::AdmitParticipant => "admit_participant",
            Self::ManageSession => "manage_session",
        }
    }
}

/// What a capability is resolved against.
///
/// `membership` is the caller's role in the target room, or `None` when they
/// have no membership. It is read only by room-scoped capabilities; a global
/// capability resolves identically whether or not it is present.
#[derive(Clone, Copy, Debug)]
pub struct CapabilityContext {
    pub membership: Option<RoomRole>,
}

impl CapabilityContext {
    /// Context for a room-scoped check.
    #[must_use]
    pub const fn in_room(membership: Option<RoomRole>) -> Self {
        Self { membership }
    }

    /// Context for a global check, which ignores membership entirely.
    #[must_use]
    pub const fn global() -> Self {
        Self { membership: None }
    }
}

/// The capabilities a principal holds in one context.
///
/// A resolved set, not a query: callers ask what was granted rather than
/// re-deriving it, and an empty set means deny.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapabilitySet {
    granted: Vec<Capability>,
}

impl CapabilitySet {
    #[must_use]
    pub fn new(granted: Vec<Capability>) -> Self {
        Self { granted }
    }

    /// An empty set. Deny-by-default is the absence of a grant.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn has(&self, capability: Capability) -> bool {
        self.granted.contains(&capability)
    }

    /// Granted capabilities, in declaration order, for introspection.
    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.granted.iter().copied()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.granted.is_empty()
    }
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

/// Resolves a principal to the capabilities it holds in a context.
///
/// Implementations must deny unless an explicit rule grants a capability, and
/// must not branch on which transport called them: REST and WebSocket share
/// one resolver so a control hidden in the UI and a request forged by hand are
/// decided identically.
pub trait CapabilityResolver: Send + Sync {
    fn resolve(&self, principal: &Principal, context: CapabilityContext) -> CapabilitySet;
}

/// Resolve capabilities and map a missing grant into a denial.
///
/// Callers that need the set itself (introspection) use the resolver directly.
#[must_use]
pub fn resolve_capabilities(
    resolver: &dyn CapabilityResolver,
    principal: &Principal,
    context: CapabilityContext,
) -> CapabilitySet {
    resolver.resolve(principal, context)
}

/// Fail unless the principal holds `capability` in `context`.
pub fn require_capability(
    resolver: &dyn CapabilityResolver,
    principal: &Principal,
    context: CapabilityContext,
    capability: Capability,
) -> Result<(), AuthorizationError> {
    if resolver.resolve(principal, context).has(capability) {
        Ok(())
    } else {
        Err(AuthorizationError::Denied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahfel_domain::{ActorId, ActorKind};
    use uuid::Uuid;

    const ROOM_SCOPED: [Capability; 7] = [
        Capability::Join,
        Capability::PublishMedia,
        Capability::ShareScreen,
        Capability::MuteOthers,
        Capability::RemoveParticipant,
        Capability::AdmitParticipant,
        Capability::ManageSession,
    ];

    const ALL: [Capability; 10] = [
        Capability::ManageSystem,
        Capability::CreateRoom,
        Capability::ViewOwnIdentity,
        Capability::Join,
        Capability::PublishMedia,
        Capability::ShareScreen,
        Capability::MuteOthers,
        Capability::RemoveParticipant,
        Capability::AdmitParticipant,
        Capability::ManageSession,
    ];

    fn principal_with_roles(roles: &[&str]) -> Principal {
        Principal {
            actor_id: ActorId::from_uuid(Uuid::nil()),
            kind: ActorKind::User,
            display_name: None,
            avatar_url: None,
            email: None,
            email_verified: false,
            username: None,
            roles: roles.iter().map(|role| (*role).to_owned()).collect(),
        }
    }

    fn service_principal() -> Principal {
        Principal {
            kind: ActorKind::Service,
            roles: Vec::new(),
            ..principal_with_roles(&[])
        }
    }

    fn has(principal: &Principal, membership: Option<RoomRole>, capability: Capability) -> bool {
        crate::policy::ServerCapabilityResolver
            .resolve(principal, CapabilityContext::in_room(membership))
            .has(capability)
    }

    #[test]
    fn capability_names_are_stable() {
        for capability in ALL {
            assert!(!capability.as_str().is_empty());
        }
        assert_eq!(Capability::ManageSession.as_str(), "manage_session");
        assert!(!Capability::ManageSystem.is_room_scoped());
        assert!(Capability::Join.is_room_scoped());
    }

    #[test]
    fn admin_holds_every_capability() {
        let admin = principal_with_roles(&["admin"]);
        let granted =
            crate::policy::ServerCapabilityResolver.resolve(&admin, CapabilityContext::global());
        for capability in ALL {
            assert!(granted.has(capability), "admin must hold {capability:?}");
        }
    }

    #[test]
    fn manager_holds_the_full_room_set_but_not_system_administration() {
        let member = principal_with_roles(&["member"]);
        assert!(has(
            &member,
            Some(RoomRole::Manager),
            Capability::ManageSession
        ));
        for capability in ROOM_SCOPED {
            assert!(
                has(&member, Some(RoomRole::Manager), capability),
                "a manager must hold {capability:?}"
            );
        }
        assert!(!has(
            &member,
            Some(RoomRole::Manager),
            Capability::ManageSystem
        ));
    }

    #[test]
    fn member_holds_only_participation_capabilities() {
        let member = principal_with_roles(&["member"]);
        assert!(has(&member, Some(RoomRole::Member), Capability::Join));
        assert!(has(
            &member,
            Some(RoomRole::Member),
            Capability::PublishMedia
        ));
        assert!(has(
            &member,
            Some(RoomRole::Member),
            Capability::ShareScreen
        ));
        for denied in [
            Capability::MuteOthers,
            Capability::RemoveParticipant,
            Capability::AdmitParticipant,
            Capability::ManageSession,
            Capability::ManageSystem,
        ] {
            assert!(
                !has(&member, Some(RoomRole::Member), denied),
                "a member must not hold {denied:?}"
            );
        }
    }

    #[test]
    fn moderator_moderates_without_managing_sessions() {
        let moderator = principal_with_roles(&["moderator"]);
        assert!(has(&moderator, None, Capability::MuteOthers));
        assert!(has(&moderator, None, Capability::RemoveParticipant));
        assert!(has(&moderator, None, Capability::AdmitParticipant));
        assert!(!has(&moderator, None, Capability::ManageSession));
        assert!(!has(&moderator, None, Capability::ManageSystem));
    }

    #[test]
    fn moderator_who_manages_a_room_also_manages_that_session() {
        let moderator = principal_with_roles(&["moderator"]);
        assert!(has(
            &moderator,
            Some(RoomRole::Manager),
            Capability::ManageSession
        ));
    }

    #[test]
    fn service_principal_holds_no_global_administration() {
        let service = service_principal();
        assert!(has(&service, None, Capability::CreateRoom));
        assert!(has(&service, None, Capability::ViewOwnIdentity));
        assert!(!has(&service, None, Capability::ManageSystem));
        assert!(!has(&service, None, Capability::MuteOthers));
    }

    #[test]
    fn service_principal_managing_a_room_holds_the_room_set() {
        let service = service_principal();
        assert!(has(
            &service,
            Some(RoomRole::Manager),
            Capability::ManageSession
        ));
        assert!(has(
            &service,
            Some(RoomRole::Manager),
            Capability::RemoveParticipant
        ));
        assert!(!has(
            &service,
            Some(RoomRole::Manager),
            Capability::ManageSystem
        ));
    }

    #[test]
    fn an_actor_without_membership_holds_only_global_grants() {
        let actor = principal_with_roles(&[]);
        assert!(has(&actor, None, Capability::CreateRoom));
        assert!(has(&actor, None, Capability::ViewOwnIdentity));
        for denied in ROOM_SCOPED {
            assert!(
                !has(&actor, None, denied),
                "an outsider must not hold {denied:?}"
            );
        }
    }

    #[test]
    fn require_capability_denies_by_default() {
        let actor = principal_with_roles(&[]);
        assert!(
            require_capability(
                &crate::policy::ServerCapabilityResolver,
                &actor,
                CapabilityContext::global(),
                Capability::CreateRoom
            )
            .is_ok()
        );
        assert_eq!(
            require_capability(
                &crate::policy::ServerCapabilityResolver,
                &actor,
                CapabilityContext::global(),
                Capability::ManageSystem
            ),
            Err(AuthorizationError::Denied)
        );
    }

    #[test]
    fn introspection_lists_only_granted_capabilities() {
        let member = principal_with_roles(&["member"]);
        let granted = crate::policy::ServerCapabilityResolver
            .resolve(&member, CapabilityContext::in_room(Some(RoomRole::Member)));
        let names: Vec<&str> = granted.iter().map(Capability::as_str).collect();
        assert!(names.contains(&"join"));
        assert!(!names.contains(&"manage_session"));
        assert!(!names.contains(&"manage_system"));
    }
}
