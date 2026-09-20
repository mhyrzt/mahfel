use anyhow::{Context, Result};
use mahfel_config::Settings;
use mahfel_domain::{NodeId, RoomId, UserId};
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
use mahfel_telemetry::{init_tracing, operations, serve_http, shutdown_signal, wait_for_shutdown};
use mediasoup::prelude::MediaKind;
use prometheus::{Counter, IntGauge, Opts};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;

const SERVICE: &str = "mahfel-media";

#[inline]
fn saturating_u64(v: usize) -> u64 {
    u64::try_from(v).unwrap_or(u64::MAX)
}

#[inline]
fn saturating_u32(v: usize) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

#[inline]
fn saturating_i64(v: usize) -> i64 {
    i64::try_from(v).unwrap_or(0)
}

#[derive(Clone)]
struct MediaControlService {
    node_id: NodeId,
    worker_count: u32,
    rtc_ports: Vec<u32>,
    media: Arc<MediaRuntime>,
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
            active_workers: saturating_u32(self.media.active_workers()),
            active_rooms: saturating_u64(self.media.room_count().await),
            participants: 0,
            transports: saturating_u64(self.media.transport_count().await),
            producers: saturating_u64(self.media.producer_count().await),
            consumers: saturating_u64(self.media.consumer_count().await),
        }))
    }

    async fn create_room(
        &self,
        request: Request<CreateRoomRequest>,
    ) -> Result<Response<CreateRoomResponse>, Status> {
        let raw = require_field(request.into_inner().room_id, "room_id")?;
        let room_id = parse_room_id(&raw)?;
        self.media.create_room(room_id).await.map_err(|e| {
            tracing::error!(%raw, error=%e, "create_room failed");
            Status::internal("router creation failed")
        })?;
        Ok(Response::new(CreateRoomResponse { room_id: raw }))
    }

    async fn delete_room(
        &self,
        request: Request<DeleteRoomRequest>,
    ) -> Result<Response<DeleteRoomResponse>, Status> {
        let raw = request.into_inner().room_id.trim().to_owned();
        if raw.is_empty() {
            return Err(Status::invalid_argument("room_id must not be empty"));
        }
        let room_id = parse_room_id(&raw)?;
        let _ = self.media.delete_room(room_id).await;
        Ok(Response::new(DeleteRoomResponse {}))
    }
    async fn create_web_rtc_transport(
        &self,
        request: Request<CreateWebRtcTransportRequest>,
    ) -> Result<Response<CreateWebRtcTransportResponse>, Status> {
        let inner = request.into_inner();
        let raw_room = require_field(inner.room_id, "room_id")?;
        let raw_participant = require_field(inner.participant_id, "participant_id")?;
        let room_id = parse_room_id(&raw_room)?;
        let participant_id = parse_user_id(&raw_participant)?;
        let (transport_id, ice_parameters, ice_candidates, dtls_parameters) = self
            .media
            .create_transport(room_id, participant_id)
            .await
            .map_err(|e| match e {
                mahfel_rtc::RtcError::RoomNotFound { .. } => Status::not_found(e.to_string()),
                _ => {
                    tracing::error!(%raw_room, %raw_participant, error=%e, "create_transport failed");
                    Status::internal("transport creation failed")
                }
            })?;
        Ok(Response::new(CreateWebRtcTransportResponse {
            transport_id,
            ice_parameters,
            ice_candidates,
            dtls_parameters,
        }))
    }

    async fn connect_web_rtc_transport(
        &self,
        request: Request<ConnectWebRtcTransportRequest>,
    ) -> Result<Response<ConnectWebRtcTransportResponse>, Status> {
        let inner = request.into_inner();
        let raw_room = require_field(inner.room_id, "room_id")?;
        let transport_id = require_field(inner.transport_id, "transport_id")?;
        let dtls_parameters = require_field(inner.dtls_parameters, "dtls_parameters")?;
        let room_id = parse_room_id(&raw_room)?;
        self.media
            .connect_transport(room_id, &transport_id, &dtls_parameters)
            .await
            .map_err(|e| match e {
                mahfel_rtc::RtcError::RoomNotFound { .. }
                | mahfel_rtc::RtcError::TransportNotFound { .. } => {
                    Status::not_found(e.to_string())
                }
                mahfel_rtc::RtcError::Dtls(_) => Status::invalid_argument(e.to_string()),
                _ => {
                    tracing::error!(%raw_room, %transport_id, error=%e, "connect_transport failed");
                    Status::internal("transport connect failed")
                }
            })?;
        Ok(Response::new(ConnectWebRtcTransportResponse {}))
    }
    async fn produce(
        &self,
        request: Request<ProduceRequest>,
    ) -> Result<Response<ProduceResponse>, Status> {
        let inner = request.into_inner();
        let raw_room = require_field(inner.room_id, "room_id")?;
        let raw_participant = require_field(inner.participant_id, "participant_id")?;
        let transport_id = require_field(inner.transport_id, "transport_id")?;
        let raw_kind = require_field(inner.kind, "kind")?;
        let rtp_parameters = require_field(inner.rtp_parameters, "rtp_parameters")?;
        let room_id = parse_room_id(&raw_room)?;
        let participant_id = parse_user_id(&raw_participant)?;
        let kind = parse_media_kind(&raw_kind)?;
        let producer_id = self
            .media
            .create_producer(
                room_id,
                &transport_id,
                participant_id,
                kind,
                &rtp_parameters,
            )
            .await
            .map_err(|e| match e {
                mahfel_rtc::RtcError::RoomNotFound { .. }
                | mahfel_rtc::RtcError::TransportNotFound { .. } => {
                    Status::not_found(e.to_string())
                }
                mahfel_rtc::RtcError::RtpParameters(_) | mahfel_rtc::RtcError::Dtls(_) => {
                    Status::invalid_argument(e.to_string())
                }
                _ => {
                    tracing::error!(%raw_room, %transport_id, error=%e, "produce failed");
                    Status::internal("producer creation failed")
                }
            })?;
        Ok(Response::new(ProduceResponse { producer_id }))
    }
    async fn consume(
        &self,
        request: Request<ConsumeRequest>,
    ) -> Result<Response<ConsumeResponse>, Status> {
        let inner = request.into_inner();
        let raw_room = require_field(inner.room_id, "room_id")?;
        let raw_participant = require_field(inner.participant_id, "participant_id")?;
        let producer_id = require_field(inner.producer_id, "producer_id")?;
        let transport_id = require_field(inner.transport_id, "transport_id")?;
        let rtp_capabilities = require_field(inner.rtp_capabilities, "rtp_capabilities")?;
        let room_id = parse_room_id(&raw_room)?;
        let participant_id = parse_user_id(&raw_participant)?;
        let (consumer_id, rtp_parameters, paused) = self
            .media
            .create_consumer(
                room_id,
                &transport_id,
                participant_id,
                &producer_id,
                &rtp_capabilities,
            )
            .await
            .map_err(|e| match e {
                mahfel_rtc::RtcError::RoomNotFound { .. }
                | mahfel_rtc::RtcError::TransportNotFound { .. }
                | mahfel_rtc::RtcError::ProducerNotFound { .. } => Status::not_found(e.to_string()),
                mahfel_rtc::RtcError::RtpCapabilities(_) => Status::invalid_argument(e.to_string()),
                _ => {
                    tracing::error!(%raw_room, %producer_id, error=%e, "consume failed");
                    Status::internal("consumer creation failed")
                }
            })?;
        Ok(Response::new(ConsumeResponse {
            consumer_id,
            rtp_parameters,
            paused,
        }))
    }

    async fn pause_consumer(
        &self,
        request: Request<PauseConsumerRequest>,
    ) -> Result<Response<PauseConsumerResponse>, Status> {
        let inner = request.into_inner();
        let raw_room = require_field(inner.room_id, "room_id")?;
        let consumer_id = require_field(inner.consumer_id, "consumer_id")?;
        let room_id = parse_room_id(&raw_room)?;
        self.media
            .pause_consumer(room_id, &consumer_id)
            .await
            .map_err(|e| match e {
                mahfel_rtc::RtcError::ConsumerNotFound { .. }
                | mahfel_rtc::RtcError::RoomNotFound { .. } => Status::not_found(e.to_string()),
                _ => {
                    tracing::error!(%raw_room, %consumer_id, error=%e, "pause_consumer failed");
                    Status::internal("pause failed")
                }
            })?;
        Ok(Response::new(PauseConsumerResponse {}))
    }

    async fn resume_consumer(
        &self,
        request: Request<ResumeConsumerRequest>,
    ) -> Result<Response<ResumeConsumerResponse>, Status> {
        let inner = request.into_inner();
        let raw_room = require_field(inner.room_id, "room_id")?;
        let consumer_id = require_field(inner.consumer_id, "consumer_id")?;
        let room_id = parse_room_id(&raw_room)?;
        self.media
            .resume_consumer(room_id, &consumer_id)
            .await
            .map_err(|e| match e {
                mahfel_rtc::RtcError::ConsumerNotFound { .. }
                | mahfel_rtc::RtcError::RoomNotFound { .. } => Status::not_found(e.to_string()),
                _ => {
                    tracing::error!(%raw_room, %consumer_id, error=%e, "resume_consumer failed");
                    Status::internal("resume failed")
                }
            })?;
        Ok(Response::new(ResumeConsumerResponse {}))
    }
    async fn close_participant(
        &self,
        _: Request<CloseParticipantRequest>,
    ) -> Result<Response<CloseParticipantResponse>, Status> {
        planned()
    }
}

fn require_field(raw: String, field: &str) -> Result<String, Status> {
    let trimmed = raw.trim().to_owned();
    if trimmed.is_empty() {
        return Err(Status::invalid_argument(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed)
}

fn parse_room_id(raw: &str) -> Result<RoomId, Status> {
    let uuid = Uuid::parse_str(raw).map_err(|_| Status::invalid_argument("invalid room_id"))?;
    Ok(RoomId::from_uuid(uuid))
}

fn parse_user_id(raw: &str) -> Result<UserId, Status> {
    let uuid =
        Uuid::parse_str(raw).map_err(|_| Status::invalid_argument("invalid participant_id"))?;
    Ok(UserId::from_uuid(uuid))
}

fn parse_media_kind(raw: &str) -> Result<MediaKind, Status> {
    match raw {
        "audio" => Ok(MediaKind::Audio),
        "video" => Ok(MediaKind::Video),
        _ => Err(Status::invalid_argument("invalid kind")),
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
    let media = Arc::new(media);
    let node_id = NodeId::new();
    let service = MediaControlService {
        node_id: node_id.clone(),
        worker_count: saturating_u32(media.active_workers()),
        rtc_ports: media.ports().iter().copied().map(u32::from).collect(),
        media: Arc::clone(&media),
    };
    let grpc_address = settings.media.grpc_bind.parse::<SocketAddr>()?;
    let operations_address = settings.media.operations_bind.parse::<SocketAddr>()?;
    let grpc_listener = tokio::net::TcpListener::bind(grpc_address).await?;
    let operations_listener = tokio::net::TcpListener::bind(operations_address).await?;
    let (operations_router, health) = operations(SERVICE, env!("CARGO_PKG_VERSION"))?;
    // Register media-specific metrics on the same registry that feeds /metrics.
    let registry = health.registry();
    let active_workers_gauge = IntGauge::with_opts(Opts::new(
        "media_active_workers",
        "Number of mediasoup workers currently not closed",
    ))?;
    registry.register(Box::new(active_workers_gauge.clone()))?;
    let worker_failures = Counter::with_opts(Opts::new(
        "media_worker_failures_total",
        "Total observed mediasoup worker failures (closed workers)",
    ))?;
    registry.register(Box::new(worker_failures.clone()))?;
    let active_rooms_gauge = IntGauge::with_opts(Opts::new(
        "media_active_rooms",
        "Number of active mediasoup routers (rooms)",
    ))?;
    registry.register(Box::new(active_rooms_gauge.clone()))?;
    let active_transports_gauge = IntGauge::with_opts(Opts::new(
        "media_active_transports",
        "Number of active WebRtcTransports",
    ))?;
    registry.register(Box::new(active_transports_gauge.clone()))?;
    let active_producers_gauge = IntGauge::with_opts(Opts::new(
        "media_active_producers",
        "Number of active producers",
    ))?;
    registry.register(Box::new(active_producers_gauge.clone()))?;
    let active_consumers_gauge = IntGauge::with_opts(Opts::new(
        "media_active_consumers",
        "Number of active consumers",
    ))?;
    registry.register(Box::new(active_consumers_gauge.clone()))?;
    active_workers_gauge.set(saturating_i64(media.active_workers()));
    active_rooms_gauge.set(saturating_i64(media.room_count().await));
    active_transports_gauge.set(saturating_i64(media.transport_count().await));
    active_producers_gauge.set(saturating_i64(media.producer_count().await));
    active_consumers_gauge.set(saturating_i64(media.consumer_count().await));

    let expected_workers = media.active_workers();
    let health_probe = health.clone();
    let media_probe = Arc::clone(&media);
    let gauge_probe = active_workers_gauge.clone();
    let failures_probe = worker_failures.clone();
    let rooms_gauge_probe = active_rooms_gauge.clone();
    let transports_gauge_probe = active_transports_gauge.clone();
    let producers_gauge_probe = active_producers_gauge.clone();
    let consumers_gauge_probe = active_consumers_gauge.clone();

    health.set_check("grpc", true).await;
    // Initial readiness requires at least one active worker.
    health
        .set_check("mediasoup_workers", media_probe.active_workers() > 0)
        .await;
    health.set_initialized(true).await;

    // Background supervision: poll every 5s, update readiness + metrics, count drops as failures.
    // Restart policy is documented in mahfel-rtc: dead workers are NOT respawned in-process.
    // The process becomes not-ready and the orchestrator restarts the container.
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let supervision = {
        let health = health_probe.clone();
        let media = Arc::clone(&media_probe);
        let mut shutdown = shutdown_rx.clone();
        tokio::spawn(async move {
            let mut last_active = media.active_workers();
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            // First tick completes immediately; skip it.
            interval.tick().await;
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let active = media.active_workers();
                        gauge_probe.set(saturating_i64(active));
                        rooms_gauge_probe.set(saturating_i64(media.room_count().await));
                        transports_gauge_probe.set(saturating_i64(media.transport_count().await));
                        producers_gauge_probe.set(saturating_i64(media.producer_count().await));
                        consumers_gauge_probe.set(saturating_i64(media.consumer_count().await));
                        if active < last_active {
                            let delta = last_active - active;
                            failures_probe.inc_by(f64::from(u32::try_from(delta).unwrap_or(0)));
                            tracing::warn!(active, expected = expected_workers, "mediasoup worker failure detected");
                        }
                        last_active = active;
                        health.set_check("mediasoup_workers", active > 0).await;
                    }
                    changed = shutdown.changed() => {
                        if changed.is_err() || *shutdown.borrow() {
                            break;
                        }
                    }
                }
            }
        })
    };

    let grpc_shutdown = shutdown_rx.clone();
    let operations_task = tokio::spawn(serve_http(
        operations_listener,
        operations_router,
        shutdown_rx,
    ));
    let grpc_task = tokio::spawn(async move {
        Server::builder()
            .add_service(MediaControlServer::new(service))
            .serve_with_incoming_shutdown(
                TcpListenerStream::new(grpc_listener),
                wait_for_shutdown(grpc_shutdown),
            )
            .await
    });

    tracing::info!(%grpc_address, %operations_address, %node_id, expected_workers, ports=?media_probe.ports(), "media service ready");
    shutdown_signal().await;
    health.set_initialized(false).await;
    let _ = shutdown_tx.send(true);
    operations_task
        .await
        .context("join operations listener")??;
    grpc_task.await.context("join gRPC listener")??;
    supervision.abort();
    drop(media_probe);
    Ok(())
}
