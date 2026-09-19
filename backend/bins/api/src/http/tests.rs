use super::{api_router, cors_layer};
use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Method, Request, StatusCode, header},
};
use tower::ServiceExt;

fn app() -> Router {
    api_router("mahfel-api-test", "0.1.0", "node-test".to_owned())
}

async fn response_body_string(resp: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .expect("body readable");
    String::from_utf8(bytes.to_vec()).expect("utf8")
}

async fn response_json(resp: axum::response::Response) -> serde_json::Value {
    let body = response_body_string(resp).await;
    serde_json::from_str(&body).expect("valid json")
}

fn error_code(json: &serde_json::Value) -> Option<&str> {
    json.get("error")
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_str())
}

#[tokio::test]
async fn status_success_has_request_id_and_security_headers() {
    let req = Request::builder()
        .uri("/v1/status")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key("x-request-id"));
    assert_eq!(
        resp.headers()
            .get("x-content-type-options")
            .map(|v| v.to_str().expect("str")),
        Some("nosniff")
    );
    assert_eq!(
        resp.headers()
            .get("x-frame-options")
            .map(|v| v.to_str().expect("str")),
        Some("DENY")
    );
    assert!(resp.headers().contains_key("referrer-policy"));

    let json = response_json(resp).await;
    assert_eq!(
        json.get("service").and_then(|v| v.as_str()),
        Some("mahfel-api-test")
    );
    assert!(json.get("version").is_some());
    assert!(json.get("node_id").is_some());
}

#[tokio::test]
async fn request_id_is_propagated_when_client_provides() {
    let req = Request::builder()
        .uri("/v1/health")
        .header("x-request-id", "test-id-123")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok()),
        Some("test-id-123")
    );
}

#[tokio::test]
async fn invalid_json_maps_to_422_envelope_not_500() {
    let req = Request::builder()
        .uri("/v1/echo")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{ not json }"))
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(resp.headers().contains_key("x-request-id"));
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("unprocessable_entity"));
}

#[tokio::test]
async fn validation_error_maps_to_422_with_envelope() {
    let req = Request::builder()
        .uri("/v1/echo")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"message":""}"#))
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("unprocessable_entity"));
}

#[tokio::test]
async fn pagination_defaults_and_limits() {
    let app = app();

    let req = Request::builder()
        .uri("/v1/echo")
        .body(Body::empty())
        .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    let pagination = json.get("pagination").expect("pagination meta");
    assert_eq!(pagination.get("page").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(
        pagination.get("per_page").and_then(|v| v.as_u64()),
        Some(20)
    );
    assert!(json.get("data").is_some());

    let req = Request::builder()
        .uri("/v1/echo?per_page=999")
        .body(Body::empty())
        .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response_json(resp).await.get("error").is_some());

    let req = Request::builder()
        .uri("/v1/echo?page=0")
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unknown_route_returns_404_envelope() {
    let req = Request::builder()
        .uri("/v1/unknown")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert!(resp.headers().contains_key("x-request-id"));
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("not_found"));
}

#[tokio::test]
async fn unversioned_unknown_route_returns_404_envelope() {
    let req = Request::builder()
        .uri("/not-an-api-route")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = response_json(resp).await;
    assert_eq!(error_code(&json), Some("not_found"));
}

#[tokio::test]
async fn cors_preflight_allows_configured_origin_and_exposes_request_id() {
    let origin = HeaderValue::from_static("http://localhost:3000");
    let app =
        api_router("mahfel-api-test", "0.1.0", "node-test".to_owned()).layer(cors_layer(origin));

    let req = Request::builder()
        .uri("/v1/status")
        .method(Method::OPTIONS)
        .header(header::ORIGIN, "http://localhost:3000")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
        .body(Body::empty())
        .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    assert_eq!(
        resp.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|v| v.to_str().ok()),
        Some("http://localhost:3000")
    );
    assert!(
        resp.headers()
            .get(header::ACCESS_CONTROL_ALLOW_METHODS)
            .is_some(),
        "preflight advertises allowed methods"
    );

    let req = Request::builder()
        .uri("/v1/status")
        .header(header::ORIGIN, "http://localhost:3000")
        .body(Body::empty())
        .expect("request");
    let resp = app.clone().oneshot(req).await.expect("response");
    let exposed = resp
        .headers()
        .get(header::ACCESS_CONTROL_EXPOSE_HEADERS)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert!(
        exposed.contains("x-request-id"),
        "exposes request id: {exposed}"
    );

    let req = Request::builder()
        .uri("/v1/status")
        .method(Method::OPTIONS)
        .header(header::ORIGIN, "http://evil.example")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
        .body(Body::empty())
        .expect("request");
    let resp = app.oneshot(req).await.expect("response");
    assert!(
        resp.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none(),
        "unconfigured origin must not be allowed"
    );
}

#[tokio::test]
async fn pagination_slice_is_correct() {
    let req = Request::builder()
        .uri("/v1/echo?page=2&per_page=2")
        .body(Body::empty())
        .expect("request");
    let resp = app().oneshot(req).await.expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let json = response_json(resp).await;
    let data = json.get("data").and_then(|d| d.as_array()).expect("array");
    assert_eq!(data.len(), 2);
    assert_eq!(
        data[0].get("message").and_then(|m| m.as_str()),
        Some("gamma")
    );
}
