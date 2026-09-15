use mediasoup::{
    prelude::{ListenInfo, Protocol, WebRtcServer, WebRtcServerListenInfos, WebRtcServerOptions},
    worker::{Worker, WorkerSettings},
    worker_manager::WorkerManager,
};
use std::net::IpAddr;
use thiserror::Error;

pub struct MediaRuntime {
    _manager: WorkerManager,
    workers: Vec<Worker>,
    _servers: Vec<WebRtcServer>,
    ports: Vec<u16>,
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
}
