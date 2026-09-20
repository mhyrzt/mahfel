mod codecs;

pub use codecs::media_codecs;

use mediasoup::{
    prelude::{
        DtlsParameters, IceCandidate, IceParameters, ListenInfo, MediaKind, Producer,
        ProducerOptions, Protocol, Router, RtpParameters, Transport, WebRtcServer,
        WebRtcServerListenInfos, WebRtcServerOptions, WebRtcTransport, WebRtcTransportOptions,
        WebRtcTransportRemoteParameters,
    },
    router::RouterOptions,
    worker::{CreateRouterError, Worker, WorkerSettings},
    worker_manager::WorkerManager,
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use thiserror::Error;
use tokio::sync::RwLock;

/// Supervision policy: workers are supervised externally.
///
/// * Each `Worker` runs as a native mediasoup subprocess. `MediaRuntime::start`
///   validates `worker_count > 0` and that `rtc_port_min..rtc_port_max`
///   provides one UDP+TCP port per worker (one `WebRtcServer` per worker).
/// * Runtime health requires `active_workers() > 0`. A background probe in
///   `mahfel-media` publishes `mediasoup_workers` readiness every 5s; if a
///   worker dies (`worker.closed() == true`) readiness flips to `not_ready`
///   and `mahfel_media_worker_failures_total` increments.
/// * No in-process auto-restart: a dead worker is not respawned inside the
///   same process to avoid leaking `Router`/`Transport` state. The container
///   orchestrator must restart the process (Docker `restart: unless-stopped`,
///   Darkube rolling restart). This keeps failure mode explicit and observable
///   via `/health/ready` and `/metrics`.
pub struct MediaRuntime {
    _manager: WorkerManager,
    workers: Vec<Worker>,
    _servers: Vec<WebRtcServer>,
    ports: Vec<u16>,
    routers: Arc<RwLock<HashMap<String, RouterEntry>>>,
    transports: Arc<RwLock<HashMap<String, TransportEntry>>>,
    producers: Arc<RwLock<HashMap<String, ProducerEntry>>>,
    next_worker: Arc<AtomicUsize>,
}

struct RouterEntry {
    router: Router,
    worker_index: usize,
}

#[allow(dead_code)]
struct TransportEntry {
    transport: WebRtcTransport,
    room_id: String,
    participant_id: String,
}

#[allow(dead_code)]
struct ProducerEntry {
    producer: Producer,
    room_id: String,
    transport_id: String,
    participant_id: String,
}

#[derive(Debug, Error)]
pub enum RtcError {
    #[error("worker count must be greater than zero")]
    NoWorkers,
    #[error("RTC port range {minimum}-{maximum} cannot provide {workers} worker ports")]
    InsufficientPorts {
        minimum: u16,
        maximum: u16,
        workers: usize,
    },
    #[error("invalid media listen address: {0}")]
    InvalidListenAddress(#[from] std::net::AddrParseError),
    #[error("mediasoup worker initialization failed: {0}")]
    Worker(#[from] std::io::Error),
    #[error("WebRTC server initialization failed: {0}")]
    WebRtcServer(#[from] mediasoup::worker::CreateWebRtcServerError),
    #[error("router creation failed: {0}")]
    Router(#[from] CreateRouterError),
    #[error("room not found: {0}")]
    RoomNotFound(String),
    #[error("transport not found: {0}")]
    TransportNotFound(String),
    #[error("transport creation failed: {0}")]
    Transport(String),
    #[error("transport connect failed: {0}")]
    Connect(String),
    #[error("invalid DTLS parameters: {0}")]
    Dtls(String),
    #[error("invalid kind: {0}")]
    Kind(String),
    #[error("invalid RTP parameters: {0}")]
    RtpParameters(String),
    #[error("producer creation failed: {0}")]
    Producer(String),
    #[error("producer not found: {0}")]
    ProducerNotFound(String),
}

impl MediaRuntime {
    pub async fn start(
        worker_count: usize,
        port_min: u16,
        port_max: u16,
        listen_ip: &str,
        announced_address: &str,
    ) -> Result<Self, RtcError> {
        if worker_count == 0 {
            return Err(RtcError::NoWorkers);
        }
        let available = usize::from(port_max.saturating_sub(port_min)) + 1;
        if port_max < port_min || available < worker_count {
            return Err(RtcError::InsufficientPorts {
                minimum: port_min,
                maximum: port_max,
                workers: worker_count,
            });
        }

        let ip: IpAddr = listen_ip.parse()?;
        let manager = WorkerManager::new();
        let mut workers = Vec::with_capacity(worker_count);
        let mut servers = Vec::with_capacity(worker_count);
        let mut ports = Vec::with_capacity(worker_count);

        for offset in 0..worker_count {
            let port = port_min
                + u16::try_from(offset).map_err(|_| RtcError::InsufficientPorts {
                    minimum: port_min,
                    maximum: port_max,
                    workers: worker_count,
                })?;
            let worker = manager.create_worker(WorkerSettings::default()).await?;
            let udp = ListenInfo {
                protocol: Protocol::Udp,
                ip,
                announced_address: Some(announced_address.to_owned()),
                expose_internal_ip: false,
                port: Some(port),
                port_range: None,
                flags: None,
                send_buffer_size: None,
                recv_buffer_size: None,
            };
            let tcp = ListenInfo {
                protocol: Protocol::Tcp,
                ip,
                announced_address: Some(announced_address.to_owned()),
                expose_internal_ip: false,
                port: Some(port),
                port_range: None,
                flags: None,
                send_buffer_size: None,
                recv_buffer_size: None,
            };
            let listen_infos = WebRtcServerListenInfos::new(udp).insert(tcp);
            let server = worker
                .create_webrtc_server(WebRtcServerOptions::new(listen_infos))
                .await?;
            ports.push(port);
            workers.push(worker);
            servers.push(server);
        }

        Ok(Self {
            _manager: manager,
            workers,
            _servers: servers,
            ports,
            routers: Arc::new(RwLock::new(HashMap::new())),
            transports: Arc::new(RwLock::new(HashMap::new())),
            producers: Arc::new(RwLock::new(HashMap::new())),
            next_worker: Arc::new(AtomicUsize::new(0)),
        })
    }

    #[must_use]
    pub fn active_workers(&self) -> usize {
        self.workers
            .iter()
            .filter(|worker| !worker.closed())
            .count()
    }

    #[must_use]
    pub fn ports(&self) -> &[u16] {
        &self.ports
    }

    /// Number of active routers (rooms).
    pub async fn room_count(&self) -> usize {
        self.routers.read().await.len()
    }

    pub async fn transport_count(&self) -> usize {
        self.transports.read().await.len()
    }

    pub async fn producer_count(&self) -> usize {
        self.producers.read().await.len()
    }

    /// Idempotent create: returns true if newly created, false if already existed.
    pub async fn create_room(&self, room_id: &str) -> Result<bool, RtcError> {
        if self.routers.read().await.contains_key(room_id) {
            return Ok(false);
        }
        let mut routers = self.routers.write().await;
        if routers.contains_key(room_id) {
            return Ok(false);
        }
        let idx = self.next_worker.fetch_add(1, Ordering::Relaxed) % self.workers.len();
        let worker = &self.workers[idx];
        let router = worker
            .create_router(RouterOptions::new(media_codecs()))
            .await?;
        routers.insert(
            room_id.to_owned(),
            RouterEntry {
                router,
                worker_index: idx,
            },
        );
        Ok(true)
    }

    /// Idempotent delete: returns true if deleted, false if not found.
    /// Dropping the Router closes the underlying mediasoup Router and all its transports.
    pub async fn delete_room(&self, room_id: &str) -> bool {
        let entry = {
            let mut routers = self.routers.write().await;
            routers.remove(room_id)
        };
        if entry.is_some() {
            // Cleanup transports and producers belonging to this room.
            let mut transports = self.transports.write().await;
            transports.retain(|_, v| v.room_id != room_id);
            let mut producers = self.producers.write().await;
            producers.retain(|_, v| v.room_id != room_id);
            true
        } else {
            false
        }
    }

    #[must_use]
    pub async fn has_room(&self, room_id: &str) -> bool {
        self.routers.read().await.contains_key(room_id)
    }

    /// Create a WebRtcTransport for a room/participant. Returns transport_id and ICE/DTLS JSON.
    pub async fn create_transport(
        &self,
        room_id: &str,
        participant_id: &str,
    ) -> Result<(String, String, String, String), RtcError> {
        let (router, worker_index) = {
            let routers = self.routers.read().await;
            let entry = routers
                .get(room_id)
                .ok_or_else(|| RtcError::RoomNotFound(room_id.to_owned()))?;
            (entry.router.clone(), entry.worker_index)
        };
        let server = self._servers[worker_index].clone();
        let options = WebRtcTransportOptions::new_with_server(server);
        // defaults already enable_udp true, enable_tcp true when server given
        let transport = router
            .create_webrtc_transport(options)
            .await
            .map_err(|e| RtcError::Transport(e.to_string()))?;

        let ice_parameters: &IceParameters = transport.ice_parameters();
        let ice_candidates: &Vec<IceCandidate> = transport.ice_candidates();
        let dtls_parameters: DtlsParameters = transport.dtls_parameters();

        let ice_parameters_json = serde_json::to_string(ice_parameters)
            .map_err(|e| RtcError::Transport(e.to_string()))?;
        let ice_candidates_json = serde_json::to_string(ice_candidates)
            .map_err(|e| RtcError::Transport(e.to_string()))?;
        let dtls_parameters_json = serde_json::to_string(&dtls_parameters)
            .map_err(|e| RtcError::Transport(e.to_string()))?;

        let transport_id = transport.id().to_string();
        let entry = TransportEntry {
            transport,
            room_id: room_id.to_owned(),
            participant_id: participant_id.to_owned(),
        };
        self.transports
            .write()
            .await
            .insert(transport_id.clone(), entry);

        Ok((
            transport_id,
            ice_parameters_json,
            ice_candidates_json,
            dtls_parameters_json,
        ))
    }

    /// Connect a transport with remote DTLS parameters (JSON from client).
    pub async fn connect_transport(
        &self,
        room_id: &str,
        transport_id: &str,
        dtls_parameters_json: &str,
    ) -> Result<(), RtcError> {
        let dtls_parameters: DtlsParameters = serde_json::from_str(dtls_parameters_json)
            .map_err(|e| RtcError::Dtls(e.to_string()))?;

        let transports = self.transports.read().await;
        let entry = transports
            .get(transport_id)
            .ok_or_else(|| RtcError::TransportNotFound(transport_id.to_owned()))?;
        if entry.room_id != room_id {
            return Err(RtcError::TransportNotFound(transport_id.to_owned()));
        }
        entry
            .transport
            .connect(WebRtcTransportRemoteParameters { dtls_parameters })
            .await
            .map_err(|e| RtcError::Connect(e.to_string()))?;
        Ok(())
    }

    /// Create a producer on a transport. Validates transport ownership and kind.
    pub async fn create_producer(
        &self,
        room_id: &str,
        transport_id: &str,
        participant_id: &str,
        kind: &str,
        rtp_parameters_json: &str,
    ) -> Result<String, RtcError> {
        let media_kind = match kind {
            "audio" => MediaKind::Audio,
            "video" => MediaKind::Video,
            _ => return Err(RtcError::Kind(kind.to_owned())),
        };
        let rtp_parameters: RtpParameters = serde_json::from_str(rtp_parameters_json)
            .map_err(|e| RtcError::RtpParameters(e.to_string()))?;

        // Validate room and transport exist and belong to participant.
        {
            let routers = self.routers.read().await;
            if !routers.contains_key(room_id) {
                return Err(RtcError::RoomNotFound(room_id.to_owned()));
            }
        }
        let transport = {
            let transports = self.transports.read().await;
            let entry = transports
                .get(transport_id)
                .ok_or_else(|| RtcError::TransportNotFound(transport_id.to_owned()))?;
            if entry.room_id != room_id || entry.participant_id != participant_id {
                return Err(RtcError::TransportNotFound(transport_id.to_owned()));
            }
            entry.transport.clone()
        };

        let producer = transport
            .produce(ProducerOptions::new(media_kind, rtp_parameters))
            .await
            .map_err(|e| RtcError::Producer(e.to_string()))?;

        let producer_id = producer.id().to_string();
        let entry = ProducerEntry {
            producer,
            room_id: room_id.to_owned(),
            transport_id: transport_id.to_owned(),
            participant_id: participant_id.to_owned(),
        };
        self.producers
            .write()
            .await
            .insert(producer_id.clone(), entry);
        Ok(producer_id)
    }
}
