use axum::http::HeaderName;

/// Response/request correlation header.
///
/// A client-supplied value is propagated; otherwise the API generates one.
pub const HEADER: &str = "x-request-id";

#[must_use]
pub fn header() -> HeaderName {
    HeaderName::from_static(HEADER)
}
