mod handlers;
mod metrics;
mod service;
mod validation;

use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_domain::NodeId;
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::media_control_service_server::MediaControlServiceServer;
use mahfel_telemetry::{init_tracing, operations, serve_http, shutdown_signal, wait_for_shutdown};
use service::MediaControlServiceImpl;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::watch;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

const SERVICE: &str = "mahfel-media";

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::load().context("load configuration")?;
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;

    let media = Arc::new(
        MediaRuntime::start(
            settings.media.worker_count,
            settings.media.rtc_port_min,
            settings.media.rtc_port_max,
            &settings.media.listen_ip,
            &settings.media.announced_address,
        )
        .await
        .context("initialize mediasoup")?,
    );

    let node_id = NodeId::new();
    let grpc_address = settings.media.grpc_bind.parse::<SocketAddr>()?;
    let operations_address = settings.media.operations_bind.parse::<SocketAddr>()?;
    let grpc_listener = tokio::net::TcpListener::bind(grpc_address).await?;
    let operations_listener = tokio::net::TcpListener::bind(operations_address).await?;
    let (operations_router, health) = operations(SERVICE, env!("CARGO_PKG_VERSION"))?;

    let metrics = metrics::MediaMetrics::register(&health)?;
    metrics.set_all(&media).await;
    let service = MediaControlServiceImpl {
        node_id: node_id.clone(),
        worker_count: validation::saturating_u32(media.active_workers()),
        rtc_ports: media.ports().iter().copied().map(u32::from).collect(),
        media: Arc::clone(&media),
        metrics: metrics.clone(),
    };
    health.set_check("grpc", true).await;
    health
        .set_check("mediasoup_workers", media.active_workers() > 0)
        .await;
    health.set_initialized(true).await;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let supervision = metrics::spawn_supervision(
        health.clone(),
        Arc::clone(&media),
        metrics,
        shutdown_rx.clone(),
    );

    let grpc_shutdown = shutdown_rx.clone();
    let operations_task = tokio::spawn(serve_http(
        operations_listener,
        operations_router,
        shutdown_rx,
    ));
    let grpc_task = tokio::spawn(async move {
        Server::builder()
            .add_service(MediaControlServiceServer::new(service))
            .serve_with_incoming_shutdown(
                TcpListenerStream::new(grpc_listener),
                wait_for_shutdown(grpc_shutdown),
            )
            .await
    });

    tracing::info!(%grpc_address, %operations_address, %node_id, ports=?media.ports(), "media service ready");
    shutdown_signal().await;
    health.set_initialized(false).await;
    let _ = shutdown_tx.send(true);
    operations_task
        .await
        .context("join operations listener")??;
    grpc_task.await.context("join gRPC listener")??;
    supervision.abort();
    Ok(())
}
