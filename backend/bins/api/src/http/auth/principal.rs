use super::verification::AuthenticationError;
use mahfel_application::ports::Principal;
use mahfel_auth::{AuthenticatedIdentity, PrincipalKind};
use mahfel_domain::{ActorId, ActorKind};
use uuid::Uuid;

/// Build the principal from verified claims.
///
/// The local actor id comes from `app_user_id`, which provisioning injects into
/// the token. The actor kind comes from [`AuthenticatedIdentity::kind`], which
/// already normalizes an absent or unknown `app_principal_kind` to `user`, so a
/// malformed claim cannot widen access. There is no database lookup here.
pub fn principal_from_claims(
    claims: &AuthenticatedIdentity,
) -> Result<Principal, AuthenticationError> {
    let actor_id = claims
        .app_user_id
        .as_deref()
        .and_then(|value| Uuid::parse_str(value).ok())
        .map(ActorId::from_uuid)
        .ok_or(AuthenticationError::NotProvisioned)?;
    let kind = match claims.kind() {
        PrincipalKind::Service => ActorKind::Service,
        PrincipalKind::User => ActorKind::User,
    };
    Ok(Principal {
        actor_id,
        kind,
        display_name: claims.display_name.clone(),
        avatar_url: claims.avatar_url.clone(),
        email: claims.email.clone(),
        email_verified: claims.email_verified,
        username: claims.username.clone(),
        roles: claims.roles.clone(),
    })
}
