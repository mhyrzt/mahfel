use crate::http::{auth::Authenticated, error::ApiError, state::AppState};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, header},
};
use mahfel_application::ports::Principal;
use mahfel_domain::ActorId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use uuid::Uuid;

pub(super) const TICKET_TTL: Duration = Duration::from_secs(10);
const TICKET_FAMILY: &str = "ws-auth-ticket";

#[derive(Serialize, utoipa::ToSchema)]
pub(crate) struct CreateTicketResponse {
    ticket: String,
    expires_in: u64,
}

#[derive(Serialize, Deserialize)]
struct TicketPrincipal {
    actor_id: ActorId,
    kind: mahfel_domain::ActorKind,
    roles: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/v1/ws/tickets",
    tag = "realtime",
    responses(
        (status = 201, description = "Short-lived, single-use WebSocket credential", body = CreateTicketResponse),
        (status = 401, description = "Authentication required"),
        (status = 503, description = "Ticket service unavailable"),
    ),
    security(("bearerAuth" = []))
)]
#[axum::debug_handler]
pub(crate) async fn create_ticket(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
) -> Result<
    (
        axum::http::StatusCode,
        HeaderMap,
        Json<CreateTicketResponse>,
    ),
    ApiError,
> {
    let store = state
        .realtime
        .auth_tickets
        .as_ref()
        .ok_or_else(|| ApiError::unavailable("WebSocket authentication unavailable"))?;
    let ticket = format!("{}.{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let hash = Sha256::digest(ticket.as_bytes());
    let id = hex_lower(&hash);
    let value = serde_json::to_vec(&TicketPrincipal {
        actor_id: principal.actor_id,
        kind: principal.kind,
        roles: principal.roles,
    })
    .map_err(|_| ApiError::unavailable("WebSocket authentication unavailable"))?;
    store
        .store_one_time(TICKET_FAMILY, &id, &value, TICKET_TTL)
        .await
        .map_err(|_| ApiError::unavailable("WebSocket authentication unavailable"))?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    Ok((
        axum::http::StatusCode::CREATED,
        headers,
        Json(CreateTicketResponse {
            ticket,
            expires_in: TICKET_TTL.as_secs(),
        }),
    ))
}

pub(super) async fn consume(
    store: Option<&mahfel_redis::RedisCoordination>,
    ticket: &str,
) -> Option<Principal> {
    if ticket.len() != 65 || !ticket.bytes().all(|b| b.is_ascii_hexdigit() || b == b'.') {
        return None;
    }
    let hash = Sha256::digest(ticket.as_bytes());
    let value = store?
        .consume_one_time(TICKET_FAMILY, &hex_lower(&hash))
        .await
        .ok()??;
    let stored: TicketPrincipal = serde_json::from_slice(&value).ok()?;
    Some(Principal {
        actor_id: stored.actor_id,
        kind: stored.kind,
        display_name: None,
        avatar_url: None,
        email: None,
        email_verified: false,
        username: None,
        roles: stored.roles,
    })
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
