use super::error::RtcError;
use super::speaker::SpeakerHub;
use super::{consumer::ConsumerMap, producer::ProducerMap, room::RoomMap, transport::TransportMap};
use mahfel_domain::{ActorId, ProducerSource, RoomId};
use mediasoup::{
    prelude::{
        ListenInfo, MediaKind, Protocol, WebRtcServer, WebRtcServerListenInfos, WebRtcServerOptions,
    },
    worker::{Worker, WorkerSettings},
    worker_manager::WorkerManager,
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, atomic::AtomicUsize},
};
use tokio::sync::RwLock;

pub struct MediaRuntime {
    _manager: WorkerManager,
    workers: Vec<Worker>,
    _servers: Vec<WebRtcServer>,
    ports: Vec<u16>,
    pub(super) rooms: RoomMap,
    pub(super) transports: TransportMap,
    pub(super) producers: ProducerMap,
    pub(super) consumers: ConsumerMap,
    pub(super) speakers: SpeakerHub,
    next_worker: Arc<AtomicUsize>,
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
            rooms: Arc::new(RwLock::new(HashMap::new())),
            transports: Arc::new(RwLock::new(HashMap::new())),
            producers: Arc::new(RwLock::new(HashMap::new())),
            consumers: Arc::new(RwLock::new(HashMap::new())),
            speakers: SpeakerHub::new(),
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

    pub async fn room_count(&self) -> usize {
        self.rooms.read().await.len()
    }

    pub async fn transport_count(&self) -> usize {
        self.transports.read().await.len()
    }

    pub async fn producer_count(&self) -> usize {
        self.producers.read().await.len()
    }

    pub async fn consumer_count(&self) -> usize {
        self.consumers.read().await.len()
    }

    pub async fn has_room(&self, room_id: RoomId) -> bool {
        self.rooms.read().await.contains_key(&room_id)
    }

    pub async fn create_room(&self, room_id: RoomId) -> Result<bool, RtcError> {
        super::room::create_room(
            &self.rooms,
            &self.workers,
            &self.next_worker,
            &self.speakers,
            room_id,
        )
        .await
    }

    pub async fn delete_room(&self, room_id: RoomId) -> bool {
        super::room::delete_room(
            &self.rooms,
            room_id,
            &self.transports,
            &self.producers,
            &self.consumers,
            &self.speakers,
        )
        .await
    }

    pub async fn clear_room_media(&self, room_id: RoomId) {
        super::room::clear_room_media(
            &self.rooms,
            room_id,
            &self.transports,
            &self.producers,
            &self.consumers,
            &self.speakers,
        )
        .await;
    }

    pub async fn create_transport(
        &self,
        room_id: RoomId,
        participant_id: ActorId,
    ) -> Result<(String, String, String, String, String), RtcError> {
        super::transport::create_transport(
            &self.rooms,
            &self._servers,
            &self.transports,
            room_id,
            participant_id,
        )
        .await
    }

    pub async fn connect_transport(
        &self,
        room_id: RoomId,
        transport_id: &str,
        dtls_parameters_json: &str,
    ) -> Result<(), RtcError> {
        super::transport::connect_transport(
            &self.transports,
            room_id,
            transport_id,
            dtls_parameters_json,
        )
        .await
    }

    pub async fn close_transport(
        &self,
        room_id: RoomId,
        transport_id: &str,
    ) -> Result<bool, RtcError> {
        super::transport::close_transport(
            &self.transports,
            &self.producers,
            &self.consumers,
            &self.speakers,
            room_id,
            transport_id,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_producer(
        &self,
        room_id: RoomId,
        transport_id: &str,
        participant_id: ActorId,
        kind: MediaKind,
        source: ProducerSource,
        rtp_parameters_json: &str,
    ) -> Result<String, RtcError> {
        super::producer::create_producer(
            &self.rooms,
            &self.transports,
            &self.producers,
            &self.speakers,
            room_id,
            transport_id,
            participant_id,
            kind,
            source,
            rtp_parameters_json,
        )
        .await
    }

    #[must_use]
    pub fn speaker_hub(&self) -> SpeakerHub {
        self.speakers.clone()
    }

    pub async fn pause_participant_source(
        &self,
        room_id: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> Result<usize, RtcError> {
        super::producer::pause_participant_source(
            &self.producers,
            &self.speakers,
            room_id,
            participant_id,
            source,
        )
        .await
    }

    pub async fn resume_participant_source(
        &self,
        room_id: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> Result<usize, RtcError> {
        super::producer::resume_participant_source(
            &self.producers,
            &self.speakers,
            room_id,
            participant_id,
            source,
        )
        .await
    }

    pub async fn pause_producer(&self, room_id: RoomId, producer_id: &str) -> Result<(), RtcError> {
        super::producer::pause_producer(&self.producers, &self.speakers, room_id, producer_id).await
    }

    pub async fn list_producers(&self, room_id: RoomId) -> Vec<super::producer::PublishedProducer> {
        super::producer::list_producers(&self.producers, room_id).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_consumer(
        &self,
        room_id: RoomId,
        transport_id: &str,
        participant_id: ActorId,
        producer_id: &str,
        rtp_capabilities_json: &str,
    ) -> Result<(String, String, bool), RtcError> {
        super::consumer::create_consumer(
            &self.rooms,
            &self.transports,
            &self.producers,
            &self.consumers,
            room_id,
            transport_id,
            participant_id,
            producer_id,
            rtp_capabilities_json,
        )
        .await
    }

    pub async fn resume_producer(
        &self,
        room_id: RoomId,
        producer_id: &str,
    ) -> Result<(), RtcError> {
        super::producer::resume_producer(&self.producers, &self.speakers, room_id, producer_id)
            .await
    }

    pub async fn close_participant(
        &self,
        room_id: RoomId,
        participant_id: ActorId,
    ) -> Result<usize, RtcError> {
        super::producer::close_participant(&self.producers, &self.speakers, room_id, participant_id)
            .await
    }

    pub async fn close_participant_source(
        &self,
        room_id: RoomId,
        participant_id: ActorId,
        source: ProducerSource,
    ) -> Result<usize, RtcError> {
        super::producer::close_participant_source(
            &self.producers,
            &self.speakers,
            room_id,
            participant_id,
            source,
        )
        .await
    }

    pub async fn pause_consumer(&self, room_id: RoomId, consumer_id: &str) -> Result<(), RtcError> {
        super::consumer::pause_consumer(&self.consumers, room_id, consumer_id).await
    }

    pub async fn resume_consumer(
        &self,
        room_id: RoomId,
        consumer_id: &str,
    ) -> Result<(), RtcError> {
        super::consumer::resume_consumer(&self.consumers, room_id, consumer_id).await
    }
}
