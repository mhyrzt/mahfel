use crate::http::{
    auth::{self, Authenticated, VerifiedToken},
    error::ApiError,
    state::AppState,
};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use mahfel_application::{
    ApplicationError,
    ports::{Action, Principal, ProvisionedIdentity},
    users::SyncIdentity,
};
use mahfel_auth::AuthenticatedIdentity;
use serde::Serialize;
use utoipa::ToSchema;

/// Attribute Keycloak maps to the `app_user_id` access-token claim.
const APP_USER_ID_ATTRIBUTE: &str = "mahfel_user_id";

/// Application profile for the authenticated user.
#[derive(Serialize, ToSchema)]
pub(crate) struct ProfileResponse {
    user_id: String,
    display_name: Option<String>,
    email: Option<String>,
    email_verified: bool,
    username: Option<String>,
    avatar_url: Option<String>,
    roles: Vec<String>,
}

impl ProfileResponse {
    /// Profile for a request whose identity is already linked.
    fn from_principal(principal: &Principal) -> Self {
        Self {
            user_id: principal.actor_id.to_string(),
            display_name: principal.display_name.clone(),
            email: principal.email.clone(),
            email_verified: principal.email_verified,
            username: principal.username.clone(),
            avatar_url: principal.avatar_url.clone(),
            roles: principal.roles.clone(),
        }
    }

    /// Profile returned by provisioning, using the canonical local user.
    fn from_provisioned(provisioned: &ProvisionedIdentity, claims: &AuthenticatedIdentity) -> Self {
        let avatar_url = provisioned
            .user
            .avatar_url
            .as_ref()
            .or(provisioned.identity.avatar_url.as_ref())
            .map(mahfel_domain::AvatarUrl::as_str)
            .map(str::to_owned);
        Self {
            user_id: provisioned.user.id.to_string(),
            display_name: Some(provisioned.user.display_name.as_str().to_owned()),
            email: claims.email.clone(),
            email_verified: claims.email_verified,
            username: claims.username.clone(),
            avatar_url,
            roles: claims.roles.clone(),
        }
    }
}

/// Provision (or resolve) the local user and link it in Keycloak.
///
/// The frontend calls this once after login. The first call for an unseen
/// `(issuer, subject)` creates the user; later calls refresh provider fields.
/// The local user id is written onto the Keycloak user so subsequent tokens
/// carry `app_user_id`. Identity is never linked by email.
#[utoipa::path(
    post,
    path = "/v1/auth/provision",
    tag = "identity",
    responses(
        (status = 200, description = "Provisioned application profile", body = ProfileResponse),
        (status = 401, description = "Authentication required"),
        (status = 503, description = "Identity provider unavailable"),
    ),
)]
pub(crate) async fn provision(
    State(state): State<AppState>,
    VerifiedToken(claims): VerifiedToken,
) -> Result<Json<ProfileResponse>, ApiError> {
    let command = SyncIdentity {
        issuer: claims.issuer.clone(),
        subject: claims.subject.clone(),
        display_name: claims.display_name.clone(),
        email: claims.email.clone(),
        email_verified: claims.email_verified,
        avatar_url: claims.avatar_url.clone(),
    };
    let provisioned = state
        .provisioner
        .provision(command)
        .await
        .map_err(|_| ApiError::internal("identity synchronization failed"))?;

    let local_user_id = provisioned.user.id.to_string();
    if claims.app_user_id.as_deref() != Some(local_user_id.as_str()) {
        state
            .linker
            .set_user_attribute(&claims.subject, APP_USER_ID_ATTRIBUTE, &local_user_id)
            .await
            .map_err(|error| match error.kind() {
                mahfel_application::ErrorKind::Unavailable => {
                    ApiError::unavailable("identity provider unavailable")
                }
                _ => ApiError::internal("identity link failed"),
            })?;
    }

    Ok(Json(ProfileResponse::from_provisioned(
        &provisioned,
        &claims,
    )))
}

/// Current application profile, resolved from the token without a database read.
#[utoipa::path(
    get,
    path = "/v1/auth/me",
    tag = "identity",
    responses(
        (status = 200, description = "Authenticated application profile", body = ProfileResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Account not provisioned"),
    ),
)]
pub(crate) async fn me(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<ProfileResponse>, ApiError> {
    auth::authorize(&state, &principal, Action::ViewOwnIdentity)?;
    Ok(Json(ProfileResponse::from_principal(&principal)))
}

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/auth/provision", post(provision))
        .route("/auth/me", get(me))
}
