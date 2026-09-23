//! Private, authenticated media-node heartbeats.

use std::{sync::Arc, time::Duration};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use mahfel_application::ports::{CoordinationError, MediaNodeRecord, MediaNodeRegistry};
use mahfel_domain::NodeId;
use mahfel_rtc_protocol::media::{HeartbeatRequest, HeartbeatResponse};
use prometheus::{IntCounterVec, IntGauge, Opts, Registry};
use sha2::Sha256;
use tonic::{Request, Response, Status};

const HEARTBEAT_TTL: Duration = Duration::from_secs(20);

#[derive(Clone)]
pub(super) struct RegistryMetrics {
    updates: IntCounterVec,
    live_nodes: IntGauge,
    draining_nodes: IntGauge,
}

impl RegistryMetrics {
    pub(super) fn register(registry: &Registry) -> Result<Self, prometheus::Error> {
        let updates = IntCounterVec::new(
            Opts::new(
                "media_registry_updates_total",
                "Media registry updates by outcome",
            ),
            &["outcome"],
        )?;
        let live_nodes = IntGauge::with_opts(Opts::new(
            "media_registry_live_nodes",
            "Live media nodes in the last successful registry read",
        ))?;
        let draining_nodes = IntGauge::with_opts(Opts::new(
            "media_registry_draining_nodes",
            "Draining media nodes in the last successful registry read",
        ))?;
        registry.register(Box::new(updates.clone()))?;
        registry.register(Box::new(live_nodes.clone()))?;
        registry.register(Box::new(draining_nodes.clone()))?;
        Ok(Self {
            updates,
            live_nodes,
            draining_nodes,
        })
    }

    fn outcome(&self, value: &'static str) {
        self.updates.with_label_values(&[value]).inc();
    }
}

#[derive(Clone)]
pub(super) struct RegistryService {
    store: Arc<dyn MediaNodeRegistry>,
    root_secret: Arc<[u8]>,
    metrics: RegistryMetrics,
}

impl RegistryService {
    pub(super) fn new(
        store: Arc<dyn MediaNodeRegistry>,
        root_secret: &str,
        metrics: RegistryMetrics,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            root_secret.len() >= 32,
            "media registry secret must be at least 32 bytes"
        );
        Ok(Self {
            store,
            root_secret: Arc::from(root_secret.as_bytes()),
            metrics,
        })
    }

    fn authenticate(&self, request: &Request<HeartbeatRequest>) -> Result<(), Status> {
        let node_id = &request.get_ref().node_id;
        if NodeId::from_deployment(node_id.clone()).is_none() {
            return Err(Status::unauthenticated("invalid media node credential"));
        }
        let bearer = request
            .metadata()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("missing media node credential"))?;
        if bearer.len() != 44 {
            return Err(Status::unauthenticated("invalid media node credential"));
        }
        let token = STANDARD
            .decode(bearer)
            .map_err(|_| Status::unauthenticated("invalid media node credential"))?;
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.root_secret)
            .map_err(|_| Status::internal("registry authentication unavailable"))?;
        mac.update(node_id.as_bytes());
        mac.verify_slice(&token)
            .map_err(|_| Status::unauthenticated("invalid media node credential"))
    }
}

#[tonic::async_trait]
impl mahfel_rtc_protocol::media::media_registry_service_server::MediaRegistryService
    for RegistryService
{
    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        if let Err(error) = self.authenticate(&request) {
            self.metrics.outcome("unauthenticated");
            return Err(error);
        }
        let heartbeat = request.into_inner();
        let node_id = NodeId::from_deployment(heartbeat.node_id)
            .ok_or_else(|| Status::invalid_argument("invalid node identity"))?;
        let url = url::Url::parse(&heartbeat.grpc_url)
            .map_err(|_| Status::invalid_argument("invalid media gRPC URL"))?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || url.port().is_none()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
            || heartbeat.max_rooms == 0
        {
            self.metrics.outcome("invalid");
            return Err(Status::invalid_argument(
                "invalid media node capacity or address",
            ));
        }
        let record = MediaNodeRecord {
            node_id,
            grpc_url: heartbeat.grpc_url,
            max_rooms: heartbeat.max_rooms,
            active_workers: heartbeat.active_workers,
            active_rooms: heartbeat.active_rooms,
            transports: heartbeat.transports,
            draining: heartbeat.draining,
        };
        if let Err(error) = self.store.heartbeat(&record, HEARTBEAT_TTL).await {
            self.metrics.outcome("unavailable");
            return Err(registry_status(error));
        }
        match self.store.live_nodes().await {
            Ok(nodes) => {
                self.metrics
                    .live_nodes
                    .set(i64::try_from(nodes.len()).unwrap_or(i64::MAX));
                self.metrics.draining_nodes.set(
                    i64::try_from(nodes.iter().filter(|node| node.draining).count())
                        .unwrap_or(i64::MAX),
                );
            }
            Err(error) => {
                self.metrics.outcome("unavailable");
                return Err(registry_status(error));
            }
        }
        self.metrics.outcome("ok");
        Ok(Response::new(HeartbeatResponse {}))
    }
}

fn registry_status(error: CoordinationError) -> Status {
    tracing::warn!(%error, "media registry access failed");
    Status::unavailable("media registry unavailable")
}
