use super::verification::AuthenticationError;
use mahfel_application::ports::Principal;
use mahfel_auth::AuthenticatedIdentity;
use mahfel_domain::UserId;
use uuid::Uuid;

/// Build the principal from verified claims.
///
/// The local user id comes from `app_user_id`, which provisioning injects into
/// the token. There is no database lookup here.
pub fn principal_from_claims(
    claims: &AuthenticatedIdentity,
) -> Result<Principal, AuthenticationError> {
    let user_id = claims
        .app_user_id
        .as_deref()
        .and_then(|value| Uuid::parse_str(value).ok())
        .map(UserId::from_uuid)
        .ok_or(AuthenticationError::NotProvisioned)?;
    Ok(Principal {
        user_id,
        display_name: claims.display_name.clone(),
        avatar_url: claims.avatar_url.clone(),
        email: claims.email.clone(),
        email_verified: claims.email_verified,
        username: claims.username.clone(),
        roles: claims.roles.clone(),
    })
}
