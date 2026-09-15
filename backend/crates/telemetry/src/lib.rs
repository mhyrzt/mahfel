use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use prometheus::{Encoder, IntGauge, Opts, Registry, TextEncoder};
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Instant,
};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct HealthState {
    inner: Arc<HealthInner>,
}

struct HealthInner {
    started_at: Instant,
    initialized: RwLock<bool>,
    checks: RwLock<BTreeMap<String, bool>>,
    ready_gauge: IntGauge,
    registry: Registry,
}

#[derive(Clone)]
struct OperationsState {
    health: HealthState,
    registry: Registry,
}

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("metrics registration failed: {0}")]
    Metrics(#[from] prometheus::Error),
    #[error("tracing subscriber initialization failed: {0}")]
    Tracing(#[from] tracing_subscriber::util::TryInitError),
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    uptime_seconds: u64,
    checks: BTreeMap<String, bool>,
}

impl HealthState {
    pub fn new(registry: &Registry) -> Result<Self, TelemetryError> {
        let ready_gauge = IntGauge::with_opts(Opts::new(
            "service_ready",
            "Whether the service can currently accept useful work",
        ))?;
        registry.register(Box::new(ready_gauge.clone()))?;

        Ok(Self {
            inner: Arc::new(HealthInner {
                started_at: Instant::now(),
                initialized: RwLock::new(false),
                checks: RwLock::new(BTreeMap::new()),
                ready_gauge,
                registry: registry.clone(),
            }),
        })
    }

    pub fn registry(&self) -> Registry {
        self.inner.registry.clone()
    }

    pub async fn set_initialized(&self, initialized: bool) {
        *self.inner.initialized.write().await = initialized;
        self.refresh_gauge().await;
    }

    pub async fn set_check(&self, name: impl Into<String>, healthy: bool) {
        self.inner.checks.write().await.insert(name.into(), healthy);
        self.refresh_gauge().await;
    }

    async fn snapshot(&self) -> (bool, BTreeMap<String, bool>) {
        let initialized = *self.inner.initialized.read().await;
        let checks = self.inner.checks.read().await.clone();
        (initialized, checks)
    }

    async fn refresh_gauge(&self) {
        let (initialized, checks) = self.snapshot().await;
        self.inner.ready_gauge.set(i64::from(
            initialized && checks.values().all(|healthy| *healthy),
        ));
    }
}

pub fn init_tracing(filter: &str) -> Result<(), TelemetryError> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_new(filter).unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
        .try_init()
        .map_err(TelemetryError::from)
}

pub fn operations(service: &str, version: &str) -> Result<(Router, HealthState), TelemetryError> {
    let labels = HashMap::from([
        ("service".to_owned(), service.to_owned()),
        ("version".to_owned(), version.to_owned()),
    ]);
    let registry = Registry::new_custom(Some("mahfel".to_owned()), Some(labels))?;
    registry.register(Box::new(
        prometheus::process_collector::ProcessCollector::for_self(),
    ))?;
    let health = HealthState::new(&registry)?;
    let state = OperationsState {
        health: health.clone(),
        registry,
    };

    let router = Router::new()
        .route("/metrics", get(metrics))
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .with_state(state);
    Ok((router, health))
}

async fn live(State(state): State<OperationsState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        uptime_seconds: state.health.inner.started_at.elapsed().as_secs(),
        checks: BTreeMap::new(),
    })
}

async fn ready(State(state): State<OperationsState>) -> (StatusCode, Json<HealthResponse>) {
    let (initialized, checks) = state.health.snapshot().await;
    let is_ready = initialized && checks.values().all(|healthy| *healthy);
    let status = if is_ready { "ready" } else { "not_ready" };
    let code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        code,
        Json(HealthResponse {
            status,
            uptime_seconds: state.health.inner.started_at.elapsed().as_secs(),
            checks,
        }),
    )
}

async fn metrics(State(state): State<OperationsState>) -> Response {
    let encoder = TextEncoder::new();
    let families = state.registry.gather();
    let mut buffer = Vec::new();
    if let Err(error) = encoder.encode(&families, &mut buffer) {
        return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response();
    }
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, encoder.format_type())],
        buffer,
    )
        .into_response()
}

pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut terminate = match signal(SignalKind::terminate()) {
            Ok(signal) => signal,
            Err(error) => {
                tracing::warn!(%error, "failed to register SIGTERM handler");
                let _ = tokio::signal::ctrl_c().await;
                return;
            }
        };
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                if let Err(error) = result {
                    tracing::warn!(%error, "failed to receive Ctrl-C");
                }
            }
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::warn!(%error, "failed to receive Ctrl-C");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn readiness_requires_initialization_and_healthy_checks() {
        let (router, health) = operations("test", "0").expect("operations setup must work");
        health.set_check("database", true).await;

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .expect("request must be valid"),
            )
            .await
            .expect("response must be available");
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        health.set_initialized(true).await;
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .expect("request must be valid"),
            )
            .await
            .expect("response must be available");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
