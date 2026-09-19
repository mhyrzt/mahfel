mod http;

use anyhow::{Context, Result};
use axum::{Router, http::HeaderValue};
use http::{api_router, cors_layer};
use mahfel_config::Settings;
use mahfel_domain::NodeId;
use mahfel_media_client::lazy as lazy_media_client;
use mahfel_postgres::{ConnectionSettings, PoolMetrics, PoolSettings, connect, migrate, ping};
use mahfel_telemetry::{HealthState, init_tracing, operations, shutdown_signal};
use std::{ffi::OsStr, net::SocketAddr, time::Duration};
use tokio::{sync::watch, task::JoinHandle};

const SERVICE: &str = "mahfel-api";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Command {
    Serve,
    Migrate,
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::load().context("load configuration")?;
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;
    let command = command().context("parse command")?;
    let pool_settings = pool_settings(&settings);
    let connection_settings = connection_settings(&settings);
    if command == Command::Migrate {
        let pool = connect(&connection_settings, pool_settings)
            .await
            .context("connect to PostgreSQL for migration")?;
        migrate(&pool).await.context("run PostgreSQL migrations")?;
        pool.close().await;
        tracing::info!("database migrations completed");
        return Ok(());
    }

    let node_id = NodeId::new();
    let (operations_router, health) = operations(SERVICE, env!("CARGO_PKG_VERSION"))?;
    let pool_metrics = PoolMetrics::register(&health.registry())?;
    let pool = connect(&connection_settings, pool_settings)
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
        .set_check("postgresql", ping(&pool).await.is_ok())
        .await;
    health.set_initialized(true).await;

    let origin = settings
        .api
        .allowed_origin
        .parse::<HeaderValue>()
        .context("parse configured browser origin")?;
    let app = api_router(SERVICE, env!("CARGO_PKG_VERSION"), node_id.to_string())
        .layer(cors_layer(origin));

    let app_address = settings.api.http_bind.parse::<SocketAddr>()?;
    let operations_address = settings.api.operations_bind.parse::<SocketAddr>()?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let app_task = serve_http(app_address, app, shutdown_rx.clone()).await?;
    let operations_task =
        serve_http(operations_address, operations_router, shutdown_rx.clone()).await?;
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

fn command() -> Result<Command> {
    let mut arguments = std::env::args_os().skip(1);
    let command = match arguments.next().as_deref() {
        None => Command::Serve,
        Some(value) if value == OsStr::new("serve") => Command::Serve,
        Some(value) if value == OsStr::new("migrate") => Command::Migrate,
        Some(value) => anyhow::bail!("unknown command: {}", value.to_string_lossy()),
    };
    if arguments.next().is_some() {
        anyhow::bail!("expected at most one command: serve or migrate");
    }
    Ok(command)
}

fn pool_settings(settings: &Settings) -> PoolSettings {
    PoolSettings {
        min_connections: settings.database.min_connections,
        max_connections: settings.database.max_connections,
        acquire_timeout: Duration::from_secs(settings.database.acquire_timeout_seconds),
        idle_timeout: Duration::from_secs(settings.database.idle_timeout_seconds),
        max_lifetime: Duration::from_secs(settings.database.max_lifetime_seconds),
    }
}

fn connection_settings(settings: &Settings) -> ConnectionSettings {
    ConnectionSettings {
        host: settings.database.host.clone(),
        port: settings.database.port,
        user: settings.database.user.clone(),
        password: settings.database.password.clone(),
        name: settings.database.name.clone(),
    }
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
    metrics: PoolMetrics,
    health: HealthState,
    mut shutdown: watch::Receiver<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    let healthy = ping(&pool).await.is_ok();
                    metrics.observe(&pool);
                    if !healthy {
                        metrics.record_probe_failure();
                    }
                    health.set_check("postgresql", healthy).await;
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
