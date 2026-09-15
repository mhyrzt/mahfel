use anyhow::{Context, Result};
use axum::{
    Json, Router,
    http::{HeaderName, HeaderValue},
    routing::get,
};
use mahfel_config::Settings;
use mahfel_database::{connect, ping};
use mahfel_domain::NodeId;
use mahfel_media_client::lazy as lazy_media_client;
use mahfel_telemetry::{HealthState, init_tracing, operations, shutdown_signal};
use serde::Serialize;
use std::{net::SocketAddr, time::Duration};
use tokio::{sync::watch, task::JoinHandle};
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

const SERVICE: &str = "mahfel-api";

#[derive(Serialize)]
struct StatusResponse {
    service: &'static str,
    version: &'static str,
    node_id: NodeId,
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::load().context("load configuration")?;
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;
    let node_id = NodeId::new();
    let pool = connect(&settings.api.database_url)
        .await
        .context("connect to PostgreSQL")?;
    let _media =
        lazy_media_client(settings.api.media_grpc_url.clone()).context("configure media client")?;

    let (operations_router, health) = operations(SERVICE, env!("CARGO_PKG_VERSION"))?;
    health.set_check("postgresql", true).await;
    health.set_initialized(true).await;

    let origin = settings
        .api
        .allowed_origin
        .parse::<HeaderValue>()
        .context("parse configured browser origin")?;
    let response_node_id = node_id.clone();
    let request_id_header = HeaderName::from_static("x-request-id");
    let app = Router::new()
        .route(
            "/api/v1/status",
            get(move || async move {
                Json(StatusResponse {
                    service: SERVICE,
                    version: env!("CARGO_PKG_VERSION"),
                    node_id: response_node_id,
                })
            }),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(origin)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid));

    let app_address = settings.api.http_bind.parse::<SocketAddr>()?;
    let operations_address = settings.api.operations_bind.parse::<SocketAddr>()?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let app_task = serve_http(app_address, app, shutdown_rx.clone()).await?;
    let operations_task =
        serve_http(operations_address, operations_router, shutdown_rx.clone()).await?;
    let probe_task = spawn_postgres_probe(pool.clone(), health.clone(), shutdown_rx);

    tracing::info!(%app_address, %operations_address, %node_id, "API service ready");
    shutdown_signal().await;
    health.set_initialized(false).await;
    let _ = shutdown_tx.send(true);
    app_task.await.context("join application listener")??;
    operations_task
        .await
        .context("join operations listener")??;
    probe_task.await.context("join PostgreSQL probe")?;
    pool.close().await;
    Ok(())
}

async fn serve_http(
    address: SocketAddr,
    router: Router,
    shutdown: watch::Receiver<bool>,
) -> Result<JoinHandle<std::io::Result<()>>> {
    let listener = tokio::net::TcpListener::bind(address).await?;
    Ok(tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(wait_for_shutdown(shutdown))
            .await
    }))
}

fn spawn_postgres_probe(
    pool: sqlx::PgPool,
    health: HealthState,
    mut shutdown: watch::Receiver<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    health.set_check("postgresql", ping(&pool).await.is_ok()).await;
                }
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
            }
        }
    })
}

async fn wait_for_shutdown(mut shutdown: watch::Receiver<bool>) {
    while !*shutdown.borrow() && shutdown.changed().await.is_ok() {}
}
