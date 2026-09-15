use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_domain::NodeId;
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::{
    CloseParticipantRequest, CloseParticipantResponse, ConnectWebRtcTransportRequest,
    ConnectWebRtcTransportResponse, ConsumeRequest, ConsumeResponse, CreateRoomRequest,
    CreateRoomResponse, CreateWebRtcTransportRequest, CreateWebRtcTransportResponse,
    DeleteRoomRequest, DeleteRoomResponse, GetNodeInfoRequest, GetNodeInfoResponse,
    GetNodeStatsRequest, GetNodeStatsResponse, PauseConsumerRequest, PauseConsumerResponse,
    ProduceRequest, ProduceResponse, ResumeConsumerRequest, ResumeConsumerResponse,
    media_control_server::{MediaControl, MediaControlServer},
};
use mahfel_telemetry::{init_tracing, operations, shutdown_signal};
use std::net::SocketAddr;
use tokio::sync::watch;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Response, Status, transport::Server};

const SERVICE: &str = "mahfel-media";

#[derive(Clone)]
struct MediaControlService {
    node_id: NodeId,
    worker_count: u32,
    rtc_ports: Vec<u32>,
}

#[tonic::async_trait]
impl MediaControl for MediaControlService {
    async fn get_node_info(
        &self,
        _request: Request<GetNodeInfoRequest>,
    ) -> Result<Response<GetNodeInfoResponse>, Status> {
        Ok(Response::new(GetNodeInfoResponse {
            node_id: self.node_id.to_string(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            worker_count: self.worker_count,
            rtc_ports: self.rtc_ports.clone(),
        }))
    }

    async fn get_node_stats(
        &self,
        _request: Request<GetNodeStatsRequest>,
    ) -> Result<Response<GetNodeStatsResponse>, Status> {
        Ok(Response::new(GetNodeStatsResponse {
            active_workers: self.worker_count,
            active_rooms: 0,
            participants: 0,
            transports: 0,
            producers: 0,
            consumers: 0,
        }))
    }

    async fn create_room(
        &self,
        _: Request<CreateRoomRequest>,
    ) -> Result<Response<CreateRoomResponse>, Status> {
        planned()
    }
    async fn delete_room(
        &self,
        _: Request<DeleteRoomRequest>,
    ) -> Result<Response<DeleteRoomResponse>, Status> {
        planned()
    }
    async fn create_web_rtc_transport(
        &self,
        _: Request<CreateWebRtcTransportRequest>,
    ) -> Result<Response<CreateWebRtcTransportResponse>, Status> {
        planned()
    }
    async fn connect_web_rtc_transport(
        &self,
        _: Request<ConnectWebRtcTransportRequest>,
    ) -> Result<Response<ConnectWebRtcTransportResponse>, Status> {
        planned()
    }
    async fn produce(
        &self,
        _: Request<ProduceRequest>,
    ) -> Result<Response<ProduceResponse>, Status> {
        planned()
    }
    async fn consume(
        &self,
        _: Request<ConsumeRequest>,
    ) -> Result<Response<ConsumeResponse>, Status> {
        planned()
    }
    async fn pause_consumer(
        &self,
        _: Request<PauseConsumerRequest>,
    ) -> Result<Response<PauseConsumerResponse>, Status> {
        planned()
    }
    async fn resume_consumer(
        &self,
        _: Request<ResumeConsumerRequest>,
    ) -> Result<Response<ResumeConsumerResponse>, Status> {
        planned()
    }
    async fn close_participant(
        &self,
        _: Request<CloseParticipantRequest>,
    ) -> Result<Response<CloseParticipantResponse>, Status> {
        planned()
    }
}

fn planned<T>() -> Result<Response<T>, Status> {
    Err(Status::unimplemented(
        "media operation is planned for milestone M3",
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::load().context("load configuration")?;
    init_tracing(&settings.telemetry.log_filter).context("initialize tracing")?;
    let media = MediaRuntime::start(
        settings.media.worker_count,
        settings.media.rtc_port_min,
        settings.media.rtc_port_max,
        &settings.media.listen_ip,
        &settings.media.announced_address,
    )
    .await
    .context("initialize mediasoup")?;
    let node_id = NodeId::new();
    let service = MediaControlService {
        node_id: node_id.clone(),
        worker_count: u32::try_from(media.active_workers())
            .context("worker count exceeds protocol range")?,
        rtc_ports: media.ports().iter().copied().map(u32::from).collect(),
    };
    let grpc_address = settings.media.grpc_bind.parse::<SocketAddr>()?;
    let operations_address = settings.media.operations_bind.parse::<SocketAddr>()?;
    let grpc_listener = tokio::net::TcpListener::bind(grpc_address).await?;
    let operations_listener = tokio::net::TcpListener::bind(operations_address).await?;
    let (operations_router, health) = operations(SERVICE, env!("CARGO_PKG_VERSION"))?;
    health
        .set_check("mediasoup_workers", media.active_workers() > 0)
        .await;
    health.set_check("grpc", true).await;
    health.set_initialized(true).await;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let grpc_shutdown = shutdown_rx.clone();
    let operations_task = tokio::spawn(async move {
        axum::serve(operations_listener, operations_router)
            .with_graceful_shutdown(wait_for_shutdown(shutdown_rx))
            .await
    });
    let grpc_task = tokio::spawn(async move {
        Server::builder()
            .add_service(MediaControlServer::new(service))
            .serve_with_incoming_shutdown(
                TcpListenerStream::new(grpc_listener),
                wait_for_shutdown(grpc_shutdown),
            )
            .await
    });

    tracing::info!(%grpc_address, %operations_address, %node_id, "media service ready");
    shutdown_signal().await;
    health.set_initialized(false).await;
    let _ = shutdown_tx.send(true);
    operations_task
        .await
        .context("join operations listener")??;
    grpc_task.await.context("join gRPC listener")??;
    drop(media);
    Ok(())
}

async fn wait_for_shutdown(mut shutdown: watch::Receiver<bool>) {
    while !*shutdown.borrow() && shutdown.changed().await.is_ok() {}
}
