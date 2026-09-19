use crate::{http, settings};
use anyhow::{Context, Result};
use axum::{Router, http::HeaderValue};
use mahfel_application::ports::{AllowAuthenticated, Principal};
use mahfel_config::Settings;
use mahfel_domain::{NodeId, UserId};
use mahfel_media_client::lazy as lazy_media_client;
use mahfel_postgres::{PoolMetrics, PostgresStores, connect, migrate, ping};
use mahfel_telemetry::{HealthState, operations, serve_http, shutdown_signal, spawn_probe};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::watch, task::JoinHandle};
use uuid::Uuid;

const SERVICE: &str = "mahfel-api";
const POSTGRES_PROBE: &str = "postgresql";
const PROBE_INTERVAL: Duration = Duration::from_secs(5);

pub(crate) async fn run(settings: Settings) -> Result<()> {
    let node_id = NodeId::new();
    let (operations_router, health) = operations(SERVICE, env!("CARGO_PKG_VERSION"))?;
    let pool_metrics = PoolMetrics::register(&health.registry())?;
    let pool = connect(&settings::connection(&settings), settings::pool(&settings))
        .await
        .context("connect to PostgreSQL")?;
    if settings.database.migrate_on_start {
        migrate(&pool)
            .await
            .context("run startup PostgreSQL migrations")?;
    }
    let _media =
        lazy_media_client(settings.api.media_grpc_url.clone()).context("configure media client")?;

    pool_metrics.observe(&pool);
    health
        .set_check(POSTGRES_PROBE, ping(&pool).await.is_ok())
        .await;
    health.set_initialized(true).await;

    let origin = settings
        .api
        .allowed_origin
        .parse::<HeaderValue>()
        .context("parse configured browser origin")?;
    let principal = settings
        .api
        .development_principal_user_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value.parse::<Uuid>().map(|id| Principal {
                user_id: UserId::from_uuid(id),
            })
        })
        .transpose()
        .context("parse development principal user id")?;
    let stores = PostgresStores::new(pool.clone());
    let state = http::AppState {
        service: SERVICE,
        version: env!("CARGO_PKG_VERSION"),
        node_id: node_id.to_string(),
        auth: http::auth::AuthState::new(principal, Arc::new(AllowAuthenticated)),
        rooms: stores.rooms,
    };
    let app = http::api_router(state).layer(http::cors_layer(origin));

    let app_address = settings.api.http_bind.parse::<SocketAddr>()?;
    let operations_address = settings.api.operations_bind.parse::<SocketAddr>()?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let app_task = spawn_listener(app_address, app, shutdown_rx.clone()).await?;
    let operations_task =
        spawn_listener(operations_address, operations_router, shutdown_rx.clone()).await?;
    let probe_task = spawn_postgres_probe(pool.clone(), pool_metrics, health.clone(), shutdown_rx);

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

async fn spawn_listener(
    address: SocketAddr,
    router: Router,
    shutdown: watch::Receiver<bool>,
) -> Result<JoinHandle<std::io::Result<()>>> {
    let listener = TcpListener::bind(address).await?;
    Ok(tokio::spawn(serve_http(listener, router, shutdown)))
}

fn spawn_postgres_probe(
    pool: sqlx::PgPool,
    metrics: PoolMetrics,
    health: HealthState,
    shutdown: watch::Receiver<bool>,
) -> JoinHandle<()> {
    spawn_probe(
        POSTGRES_PROBE,
        PROBE_INTERVAL,
        health,
        shutdown,
        move || {
            let pool = pool.clone();
            let metrics = metrics.clone();
            async move {
                let healthy = ping(&pool).await.is_ok();
                metrics.observe(&pool);
                if !healthy {
                    metrics.record_probe_failure();
                }
                healthy
            }
        },
    )
}
