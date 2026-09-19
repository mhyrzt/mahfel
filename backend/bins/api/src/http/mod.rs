pub mod cors;
pub mod error;
pub mod extractor;
pub mod middleware;
pub mod pagination;
pub mod request_id;
mod routes;

pub use cors::cors_layer;

use axum::{Router, middleware as axum_middleware};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

/// Build the versioned API router mounted at `/v1`.
///
/// Composition only:
/// - routes: versioned route table plus JSON 404 fallback
/// - error handling: invalid JSON/query/pagination never becomes 500
/// - request IDs: `x-request-id` set + propagated + traced
/// - security headers: applied to every response
///
/// CORS is layered by the composition root via [`cors_layer`] because it
/// depends on configured deployment origin.
pub fn api_router(service: &'static str, version: &'static str, node_id: String) -> Router {
    let app = routes::router(service, version, node_id)
        .layer(axum_middleware::from_fn(middleware::security_headers));

    app.layer(PropagateRequestIdLayer::new(request_id::header()))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::new(
            request_id::header(),
            MakeRequestUuid,
        ))
}

#[cfg(test)]
mod tests;
