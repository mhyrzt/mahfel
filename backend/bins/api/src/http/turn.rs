use crate::http::{
    auth::Authenticated,
    state::{AppState, RoomStoreState},
};
use axum::{Json, Router, extract::State, routing::get};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha1::Sha1;
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TurnCredentialsResponse {
    pub username: String,
    pub password: String,
    pub ttl: u64,
    pub uris: Vec<String>,
}

/// Short-lived TURN credentials (HMAC-SHA1, coturn static-auth-secret).
///
/// Username is `<expiry>:<user_id>` where expiry is Unix timestamp.
/// Password is `base64(hmac_sha1(secret, username))`.
/// Valid for `turn.ttl_seconds` (default 300s). Requires `Authenticated`.
#[utoipa::path(
    get,
    path = "/v1/turn/credentials",
    tag = "turn",
    responses(
        (status = 200, description = "TURN credentials", body = TurnCredentialsResponse),
        (status = 401, description = "unauthorized"),
    ),
    security(("bearerAuth" = []))
)]
pub async fn turn_credentials<R: crate::http::state::RoomStoreState>(
    State(state): State<AppState<R>>,
    Authenticated(principal): Authenticated,
) -> Json<TurnCredentialsResponse> {
    let ttl = state.turn.ttl_seconds;
    let expiry = OffsetDateTime::now_utc().unix_timestamp() as u64 + ttl;
    let username = format!("{}:{}", expiry, principal.user_id);
    let mut mac = Hmac::<Sha1>::new_from_slice(state.turn.static_secret.as_bytes())
        .expect("HMAC key must be valid");
    mac.update(username.as_bytes());
    let password = BASE64_STANDARD.encode(mac.finalize().into_bytes());

    Json(TurnCredentialsResponse {
        username,
        password,
        ttl,
        uris: state.turn.uris.clone(),
    })
}

pub(crate) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new().route("/turn/credentials", get(turn_credentials::<R>))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    #[test]
    fn hmac_is_deterministic() {
        let secret = "test-secret";
        let username = "1234567890:alice";
        let mut mac1 = Hmac::<Sha1>::new_from_slice(secret.as_bytes()).expect("hmac");
        mac1.update(username.as_bytes());
        let p1 = STANDARD.encode(mac1.finalize().into_bytes());
        let mut mac2 = Hmac::<Sha1>::new_from_slice(secret.as_bytes()).expect("hmac");
        mac2.update(username.as_bytes());
        let p2 = STANDARD.encode(mac2.finalize().into_bytes());
        assert_eq!(p1, p2);
    }
}
