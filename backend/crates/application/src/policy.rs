//! Session and room capability policy.
//!
//! One seam resolves an authenticated principal plus a room context into a set
//! of capabilities, so REST and WebSocket decide identically. Capabilities
//! replace a fixed vertical role enum: a vertical maps its vocabulary onto
//! capabilities instead of adding roles (ADR 0020).
//!
//! Deny-by-default holds at every step: global roles are matched against the
//! verified claim set, room grants come from the resolved membership, and a
//! capability that no rule names is not granted.

use crate::ports::{
    AuthorizationError, Capability, CapabilityContext, CapabilityResolver, CapabilitySet, Principal,
};
use mahfel_domain::RoleCapability;

/// Global role that grants full system administration.
///
/// A scope name from the verified claim set, not a vertical role: it names what
/// the role may do rather than who the actor is.
pub const ADMIN_ROLE: &str = "admin";

/// Global role that moderates any room.
pub const MODERATOR_ROLE: &str = "moderator";

/// Server-owned policy used in production.
///
/// Lives in the application crate rather than an adapter because it is business
/// policy, not infrastructure: transports call it through the [`CapabilityResolver`]
/// port, and tests substitute their own implementation.
///
/// [`CapabilityResolver`]: crate::ports::CapabilityResolver
#[derive(Clone, Copy, Debug, Default)]
pub struct ServerCapabilityResolver;

impl ServerCapabilityResolver {
    fn holds(principal: &Principal, role: &str) -> bool {
        principal
            .roles
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(role))
    }

    fn is_admin(principal: &Principal) -> bool {
        Self::holds(principal, ADMIN_ROLE)
    }

    fn is_moderator(principal: &Principal) -> bool {
        Self::holds(principal, MODERATOR_ROLE)
    }

    /// Capabilities every authenticated actor holds regardless of room.
    fn global_grants(principal: &Principal) -> Vec<Capability> {
        let mut granted = vec![Capability::CreateRoom, Capability::ViewOwnIdentity];
        if Self::is_admin(principal) {
            granted.push(Capability::ManageSystem);
        }
        granted
    }

    /// Capabilities granted by the caller's role in one room.
    ///
    /// The domain owns the role-to-capability mapping; this only names the
    /// transport-facing capabilities each role capability corresponds to.
    fn room_grants(membership: Option<mahfel_domain::RoomRole>) -> Vec<Capability> {
        let Some(role) = membership else {
            return Vec::new();
        };
        RoleCapability::for_role(role)
            .iter()
            .flat_map(|capability| Self::to_capabilities(*capability))
            .collect()
    }

    fn to_capabilities(capability: RoleCapability) -> Vec<Capability> {
        match capability {
            // Participating means entering and contributing media.
            RoleCapability::Participate | RoleCapability::JoinSession => vec![
                Capability::Join,
                Capability::PublishMedia,
                Capability::ShareScreen,
            ],
            RoleCapability::Moderate => vec![
                Capability::MuteOthers,
                Capability::RemoveParticipant,
                Capability::AdmitParticipant,
            ],
            // Managing a room includes managing its sessions.
            RoleCapability::Manage | RoleCapability::ManageSession => {
                vec![Capability::ManageSession, Capability::ManageRecording]
            }
        }
    }

    /// What managing a room grants inside one session.
    fn participant_capabilities() -> Vec<Capability> {
        vec![
            Capability::Join,
            Capability::PublishMedia,
            Capability::ShareScreen,
            Capability::MuteOthers,
            Capability::RemoveParticipant,
            Capability::AdmitParticipant,
            Capability::ManageSession,
            Capability::ManageRecording,
        ]
    }
}

impl crate::ports::CapabilityResolver for ServerCapabilityResolver {
    fn resolve(&self, principal: &Principal, context: CapabilityContext) -> CapabilitySet {
        let mut granted = Self::global_grants(principal);

        // A global moderator moderates any room, but does not manage sessions
        // unless they also hold the room's manager role.
        if Self::is_moderator(principal) {
            granted.extend([
                Capability::MuteOthers,
                Capability::RemoveParticipant,
                Capability::AdmitParticipant,
                Capability::ManageRecording,
            ]);
        }

        granted.extend(Self::room_grants(context.membership));

        if Self::is_admin(principal) {
            granted.extend(Self::participant_capabilities());
        }

        granted.sort_by_key(|capability| *capability as u8);
        granted.dedup();
        CapabilitySet::new(granted)
    }
}

/// Load the caller's membership, then resolve their capabilities in that room.
pub async fn resolve_room_capabilities<S>(
    memberships: &S,
    principal: &Principal,
    room_id: mahfel_domain::RoomId,
) -> Result<CapabilitySet, AuthorizationError>
where
    S: crate::ports::MembershipStore + ?Sized,
{
    let membership = memberships
        .role_of(room_id, principal.actor_id)
        .await
        .map_err(|_| AuthorizationError::Denied)?;
    Ok(ServerCapabilityResolver.resolve(principal, CapabilityContext::in_room(membership)))
}

/// Fail unless the caller holds `capability` in `room_id`.
///
/// Resolves against the caller's own membership, so a caller cannot supply a
/// role it does not have.
pub async fn require_room_capability<S>(
    memberships: &S,
    principal: &Principal,
    room_id: mahfel_domain::RoomId,
    capability: Capability,
) -> Result<(), AuthorizationError>
where
    S: crate::ports::MembershipStore + ?Sized,
{
    let granted = resolve_room_capabilities(memberships, principal, room_id).await?;
    if granted.has(capability) {
        return Ok(());
    }
    Err(AuthorizationError::Denied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{BoxFuture, Membership, MembershipStore, StoreError};
    use mahfel_domain::{ActorId, ActorKind, RoomId, RoomRole};
    use uuid::Uuid;

    /// Membership lookup backed by an in-memory role, so the tests exercise
    /// resolution against a real port rather than a hand-built context.
    struct FakeMemberships(Option<RoomRole>);

    impl MembershipStore for FakeMemberships {
        fn role_of(
            &self,
            _room_id: RoomId,
            _actor_id: ActorId,
        ) -> BoxFuture<'_, Result<Membership, StoreError>> {
            Box::pin(async move { Ok(self.0) })
        }
    }

    fn principal_with(roles: &[&str], kind: ActorKind) -> Principal {
        Principal {
            actor_id: ActorId::from_uuid(Uuid::nil()),
            kind,
            display_name: None,
            avatar_url: None,
            email: None,
            email_verified: false,
            username: None,
            roles: roles.iter().map(|role| (*role).to_owned()).collect(),
        }
    }

    async fn resolved(membership: Option<RoomRole>, roles: &[&str]) -> CapabilitySet {
        let memberships = FakeMemberships(membership);
        let principal = principal_with(roles, ActorKind::User);
        resolve_room_capabilities(&memberships, &principal, RoomId::new())
            .await
            .expect("resolution never fails for a readable membership")
    }

    #[tokio::test]
    async fn member_resolves_participation_only() {
        let granted = resolved(Some(RoomRole::Member), &[]).await;
        assert!(granted.has(Capability::Join));
        assert!(granted.has(Capability::PublishMedia));
        assert!(!granted.has(Capability::MuteOthers));
        assert!(!granted.has(Capability::ManageSession));
    }

    #[tokio::test]
    async fn manager_resolves_the_full_room_set() {
        let granted = resolved(Some(RoomRole::Manager), &[]).await;
        for capability in [
            Capability::Join,
            Capability::PublishMedia,
            Capability::ShareScreen,
            Capability::MuteOthers,
            Capability::RemoveParticipant,
            Capability::AdmitParticipant,
            Capability::ManageSession,
            Capability::ManageRecording,
        ] {
            assert!(granted.has(capability), "manager must hold {capability:?}");
        }
        assert!(!granted.has(Capability::ManageSystem));
    }

    #[tokio::test]
    async fn moderator_resolves_moderation_without_managing_the_session() {
        let granted = resolved(None, &[MODERATOR_ROLE]).await;
        assert!(granted.has(Capability::MuteOthers));
        assert!(!granted.has(Capability::ManageSession));
        assert!(!granted.has(Capability::ManageSystem));
    }

    #[tokio::test]
    async fn admin_resolves_everything() {
        let granted = resolved(None, &[ADMIN_ROLE]).await;
        assert!(granted.has(Capability::ManageSystem));
        assert!(granted.has(Capability::ManageSession));
    }

    #[tokio::test]
    async fn service_principal_resolves_membership_without_system_administration() {
        let memberships = FakeMemberships(Some(RoomRole::Manager));
        let principal = principal_with(&[], ActorKind::Service);
        let granted = resolve_room_capabilities(&memberships, &principal, RoomId::new())
            .await
            .expect("resolution");
        assert!(granted.has(Capability::ManageSession));
        assert!(!granted.has(Capability::ManageSystem));
    }

    #[tokio::test]
    async fn outsider_resolves_only_global_grants() {
        let granted = resolved(None, &[]).await;
        assert!(granted.has(Capability::CreateRoom));
        assert!(!granted.has(Capability::Join));
        assert!(!granted.has(Capability::PublishMedia));
    }

    #[tokio::test]
    async fn require_room_capability_denies_without_a_grant() {
        let memberships = FakeMemberships(None);
        let principal = principal_with(&[], ActorKind::User);
        let room = RoomId::new();
        assert_eq!(
            require_room_capability(&memberships, &principal, room, Capability::Join).await,
            Err(AuthorizationError::Denied)
        );
    }

    #[tokio::test]
    async fn a_non_member_may_not_publish_even_though_a_public_room_admits_them() {
        // The publish gate. A public room lets a non-member in through the join
        // guard, which checks visibility rather than capability, so the
        // capability set is what has to refuse publishing.
        let memberships = FakeMemberships(None);
        let principal = principal_with(&[], ActorKind::User);
        let room = RoomId::new();

        assert_eq!(
            require_room_capability(&memberships, &principal, room, Capability::PublishMedia).await,
            Err(AuthorizationError::Denied)
        );
        assert_eq!(
            require_room_capability(&memberships, &principal, room, Capability::ShareScreen).await,
            Err(AuthorizationError::Denied),
            "a screen share is a separate capability from speaking"
        );
    }

    #[tokio::test]
    async fn a_member_may_publish_and_share_a_screen() {
        // Guards the gate against over-refusing: both capabilities come from
        // participation, so a plain member keeps them.
        let memberships = FakeMemberships(Some(RoomRole::Member));
        let principal = principal_with(&[], ActorKind::User);
        let room = RoomId::new();

        require_room_capability(&memberships, &principal, room, Capability::PublishMedia)
            .await
            .expect("a member may publish");
        require_room_capability(&memberships, &principal, room, Capability::ShareScreen)
            .await
            .expect("a member may share a screen");
    }

    #[tokio::test]
    async fn a_demoted_manager_keeps_every_media_capability() {
        // Why the reconcile is a no-op on demotion: the media capabilities come
        // from participation, which both roles grant.
        let memberships = FakeMemberships(Some(RoomRole::Member));
        let principal = principal_with(&[], ActorKind::User);
        let room = RoomId::new();
        let granted = resolve_room_capabilities(&memberships, &principal, room)
            .await
            .expect("resolve");

        assert!(granted.has(Capability::PublishMedia));
        assert!(granted.has(Capability::ShareScreen));
        assert!(
            !granted.has(Capability::MuteOthers),
            "a demoted manager loses moderation, not media"
        );
    }
}
