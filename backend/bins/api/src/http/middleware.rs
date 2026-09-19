use axum::{
    body::Body,
    http::{HeaderName, HeaderValue, Request, header},
    middleware::Next,
    response::Response,
};

const SECURITY_HEADERS: [(&str, &str); 4] = [
    ("x-content-type-options", "nosniff"),
    ("x-frame-options", "DENY"),
    ("referrer-policy", "strict-origin-when-cross-origin"),
    (
        "permissions-policy",
        "geolocation=(), microphone=(), camera=()",
    ),
];

/// Baseline security headers applied to every response.
///
/// Existing response headers are preserved; these only fill gaps.
pub async fn security_headers(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    for (name, value) in SECURITY_HEADERS {
        let name = HeaderName::from_static(name);
        if !headers.contains_key(&name) {
            headers.insert(name, HeaderValue::from_static(value));
        }
    }

    // Legacy browser header; explicitly disabled by convention.
    if !headers.contains_key(header::X_XSS_PROTECTION) {
        headers.insert(header::X_XSS_PROTECTION, HeaderValue::from_static("0"));
    }

    response
}
