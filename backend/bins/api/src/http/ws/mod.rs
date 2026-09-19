mod auth;
mod config;
mod connection;
mod envelope;
mod metrics;
mod registry;
mod state;

pub use auth::{DevAuthenticator, UpgradeAuthenticator};
pub use config::WsConfig;
pub use metrics::WsMetrics;
pub use state::WsState;

use crate::http::{
    error::ApiError,
    state::{AppState, RoomStoreState},
};
use axum::{
    Router,
    extract::{State, WebSocketUpgrade},
    http::{HeaderMap, header},
    response::Response,
    routing::get,
};

const AT_CAPACITY: &str = "realtime capacity reached";

/// Versioned WebSocket route. The protocol version lives in the envelope, not
/// the URL.
pub(super) fn route<R: RoomStoreState>() -> Router<AppState<R>> {
    Router::new().route("/ws", get(upgrade::<R>))
}

async fn upgrade<R: RoomStoreState>(
    State(state): State<AppState<R>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let realtime = state.realtime.clone();

    let header_principal = match bearer_token(&headers) {
        Some(token) => Some(
            realtime
                .authenticator
                .authenticate(Some(token))
                .map_err(|_| ApiError::unauthorized("authentication required"))?,
        ),
        None => None,
    };

    let Some(guard) = realtime.registry.try_acquire() else {
        return Err(ApiError::unavailable(AT_CAPACITY));
    };

    let max_message_bytes = realtime.config.max_message_bytes;
    Ok(ws
        .max_message_size(max_message_bytes)
        .max_frame_size(max_message_bytes)
        .on_upgrade(move |socket| async move {
            connection::run(socket, realtime, guard, header_principal).await;
        }))
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
}
