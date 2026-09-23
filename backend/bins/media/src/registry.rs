//! Periodic capacity publication to the API's private registry service.

use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use mahfel_config::MediaSettings;
use mahfel_domain::NodeId;
use mahfel_rtc::MediaRuntime;
use mahfel_rtc_protocol::media::{
    HeartbeatRequest, media_registry_service_client::MediaRegistryServiceClient,
};
use prometheus::{IntCounter, Opts, Registry};
use tokio::sync::watch;
use tonic::{Request, metadata::MetadataValue, transport::Channel};

use crate::validation::{saturating_u32, saturating_u64};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub(crate) struct RegistryPublisher {
    client: MediaRegistryServiceClient<Channel>,
    authorization: MetadataValue<tonic::metadata::Ascii>,
    node_id: NodeId,
    grpc_url: String,
    max_rooms: u32,
    media: Arc<MediaRuntime>,
    published: IntCounter,
    failed: IntCounter,
}

impl RegistryPublisher {
    pub(crate) fn new(
        settings: &MediaSettings,
        node_id: NodeId,
        media: Arc<MediaRuntime>,
        registry: &Registry,
    ) -> Result<Self> {
        anyhow::ensure!(settings.max_rooms > 0, "media max_rooms must be positive");
        let authorization = format!("Bearer {}", settings.registry_token)
            .parse::<MetadataValue<tonic::metadata::Ascii>>()
            .context("invalid media registry credential")?;
        let client = MediaRegistryServiceClient::new(
            tonic::transport::Endpoint::from_shared(settings.registry_url.clone())?.connect_lazy(),
        );
        let published = IntCounter::with_opts(Opts::new(
            "media_registry_heartbeats_total",
            "Successful media registry heartbeats",
        ))?;
        let failed = IntCounter::with_opts(Opts::new(
            "media_registry_heartbeat_failures_total",
            "Failed media registry heartbeats",
        ))?;
        registry.register(Box::new(published.clone()))?;
        registry.register(Box::new(failed.clone()))?;
        Ok(Self {
            client,
            authorization,
            node_id,
            grpc_url: settings.grpc_advertise_url.clone(),
            max_rooms: settings.max_rooms,
            media,
            published,
            failed,
        })
    }

    async fn heartbeat(&self, draining: bool) -> Result<()> {
        let mut client = self.client.clone();
        let mut request = Request::new(HeartbeatRequest {
            node_id: self.node_id.to_string(),
            grpc_url: self.grpc_url.clone(),
            max_rooms: self.max_rooms,
            active_workers: saturating_u32(self.media.active_workers()),
            active_rooms: saturating_u64(self.media.room_count().await),
            transports: saturating_u64(self.media.transport_count().await),
            draining,
        });
        request
            .metadata_mut()
            .insert("authorization", self.authorization.clone());
        tokio::time::timeout(HEARTBEAT_TIMEOUT, client.heartbeat(request))
            .await
            .context("media registry heartbeat timed out")??;
        Ok(())
    }

    pub(crate) async fn publish(&self, draining: bool) {
        match self.heartbeat(draining).await {
            Ok(()) => {
                self.published.inc();
            }
            Err(error) => {
                self.failed.inc();
                tracing::warn!(%error, "media registry heartbeat failed");
            }
        }
    }

    pub(crate) fn spawn(
        self,
        draining: bool,
        mut shutdown: watch::Receiver<bool>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = interval.tick() => self.publish(draining).await,
                    changed = shutdown.changed() => {
                        if changed.is_err() || *shutdown.borrow() { break; }
                    }
                }
            }
        })
    }
}
