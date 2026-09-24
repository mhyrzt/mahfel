mod dto;
pub(crate) mod handlers;
pub(crate) mod presentation;

pub(crate) use dto::{
    CollaborationDocumentsResponse, CollaborationEventsResponse, EditGrantRequest, EventResponse,
    ParticipantGrantsResponse, PresentationRequest, ViewportRequest,
};
pub(crate) use handlers::{documents, events, grants, set_grant, update_viewport};
pub(crate) use presentation::{PresentationStateResponse, get_presentation, update_presentation};

use crate::http::state::AppState;
use axum::{
    Router,
    routing::{get, put},
};

pub(crate) fn route() -> Router<AppState> {
    Router::new()
        .route("/sessions/{session_id}/collaboration", get(documents))
        .route("/sessions/{session_id}/collaboration/events", get(events))
        .route("/sessions/{session_id}/collaboration/grants", get(grants))
        .route(
            "/sessions/{session_id}/collaboration/viewport",
            axum::routing::post(update_viewport),
        )
        .route(
            "/sessions/{session_id}/collaboration/presentation",
            get(get_presentation).post(update_presentation),
        )
        .route(
            "/sessions/{session_id}/collaboration/grants/{actor_id}",
            put(set_grant),
        )
}
