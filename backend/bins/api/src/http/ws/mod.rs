pub(crate) mod asyncapi;
mod chat_fanout;
mod collaboration;
mod config;
mod connection;
mod envelope;
mod metrics;
mod placement;
mod registry;
mod session_fanout;
mod speaker;
mod state;
pub(crate) mod tickets;

pub use config::WsConfig;
pub(crate) use envelope::ServerMessage;
pub use metrics::WsMetrics;
pub use placement::MediaPlacementRouter;
pub(crate) use placement::media_room_id;
#[cfg(test)]
pub(super) use registry::Outbound;
pub use state::{ParticipationStores, WsState};

use crate::http::{error::ApiError, state::AppState};
use axum::{
    Router,
    extract::{State, WebSocketUpgrade},
    http::{HeaderMap, header},
    response::Response,
    routing::{get, post},
};

const AT_CAPACITY: &str = "realtime capacity reached";

/// Versioned WebSocket route. The protocol version lives in the envelope, not
/// the URL.
pub(super) fn route() -> Router<AppState> {
    Router::new()
        .route("/ws", get(upgrade))
        .route("/ws/tickets", post(tickets::create_ticket))
        .merge(collaboration::route())
}

async fn upgrade(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let realtime = state.realtime.clone();

    let header_principal = match bearer_token(&headers) {
        Some(token) => {
            let claims = realtime
                .authenticator
                .verify(Some(token))
                .await
                .map_err(|_| ApiError::unauthorized("authentication required"))?;
            Some(
                crate::http::auth::principal_from_claims(&claims)
                    .map_err(|_| ApiError::forbidden("account is not provisioned"))?,
            )
        }
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

mod interaction_fanout;
