//! Redeem an invite link.

use crate::http::{auth::Authenticated, error::ApiError, source_address, state::AppState};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
};
use mahfel_application::invite::{RedeemInviteLink, Redeemed, redeem_invite_link};
use mahfel_domain::InviteTargetKind;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RedeemInviteLinkRequest {
    /// The code as typed or pasted. Case and separators are ignored.
    pub(crate) code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct RedeemInviteLinkResponse {
    pub(crate) target_kind: String,
    pub(crate) target_id: String,
    /// `already_member` needs no decision; `waiting` needs one.
    pub(crate) outcome: String,
    /// Present while waiting, so the client can poll its state.
    pub(crate) request_id: Option<String>,
}

/// Redeem an invite link.
///
/// A code is a bearer capability, so every unusable code returns the same
/// `404`, and attempts are rate-limited per principal and source address. The
/// code is carried in the body rather than the path: a path segment would be
/// recorded by access logs and `Referer` headers, which defeats the log hygiene
/// ADR 0021 requires.
#[utoipa::path(
    post,
    path = "/v1/invite-links/redeem",
    tag = "invite",
    request_body = RedeemInviteLinkRequest,
    responses(
        (status = 200, description = "Code resolved", body = RedeemInviteLinkResponse),
        (status = 401, description = "Authentication required"),
        (status = 404, description = "Code is unknown, expired, revoked, or the rate limit was reached"),
        (status = 409, description = "The target's lobby is at capacity"),
        (status = 503, description = "The rate limiter is unavailable"),
    ),
)]
pub(crate) async fn redeem(
    State(state): State<AppState>,
    Authenticated(principal): Authenticated,
    peer: source_address::PeerAddress,
    headers: axum::http::HeaderMap,
    payload: Result<Json<RedeemInviteLinkRequest>, JsonRejection>,
) -> Result<Json<RedeemInviteLinkResponse>, ApiError> {
    let Json(request) = payload.map_err(|_| ApiError::unprocessable("invalid request body"))?;

    // Only honoured when the peer is the configured proxy; otherwise a forged
    // header would buy a fresh budget.
    let forwarded = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok());
    let source_address = peer
        .0
        .and_then(|peer| state.source_address.resolve(peer, forwarded));

    let redeemed = redeem_invite_link(
        state.invites.as_ref(),
        state.admissions.as_ref(),
        state.sessions.as_ref(),
        state.rooms.as_ref(),
        state.code_index.as_ref(),
        &mahfel_application::invite::AttemptLimiter::default(),
        RedeemInviteLink {
            actor: principal.actor_id,
            code: request.code,
            source_address,
        },
        OffsetDateTime::now_utc(),
    )
    .await?;
    Ok(Json(RedeemInviteLinkResponse::from(redeemed)))
}

impl From<Redeemed> for RedeemInviteLinkResponse {
    fn from(redeemed: Redeemed) -> Self {
        let target = redeemed.target();
        let (outcome, request_id) = match redeemed {
            Redeemed::AlreadyMember { .. } => ("already_member", None),
            Redeemed::Waiting { request_id, .. } => ("waiting", Some(request_id.to_string())),
        };
        Self {
            target_kind: match target.kind() {
                InviteTargetKind::Room => "room",
                InviteTargetKind::Session => "session",
            }
            .to_owned(),
            target_id: target.id().to_string(),
            outcome: outcome.to_owned(),
            request_id,
        }
    }
}
