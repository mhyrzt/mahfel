//! Subscribe to the private media stream with bounded reconnects.

use super::SpeakerProjection;
use crate::http::ws::{placement::MediaPlacementRouter, registry::ConnectionRegistry};
use mahfel_application::ports::{ParticipantStore, SessionStore};
use mahfel_domain::NodeId;
use mahfel_media_client::GrpcMediaGateway;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::watch;

impl SpeakerProjection {
    pub fn spawn_placed(
        &self,
        router: Arc<MediaPlacementRouter>,
        registry: ConnectionRegistry,
        participants: Arc<dyn ParticipantStore>,
        sessions: Arc<dyn SessionStore>,
        mut shutdown: watch::Receiver<bool>,
    ) {
        let projection = self.clone();
        tokio::spawn(async move {
            let mut watchers: HashMap<NodeId, (String, tokio::task::JoinHandle<()>)> =
                HashMap::new();
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            loop {
                tokio::select! {
                    changed = shutdown.changed() => {
                        if changed.is_err() || *shutdown.borrow() { break; }
                    }
                    _ = tick.tick() => {
                        let nodes = match router.live_nodes().await {
                            Ok(nodes) => nodes,
                            Err(error) => {
                                tracing::debug!(?error, "speaker node discovery unavailable");
                                continue;
                            }
                        };
                        let live: HashMap<_, _> = nodes.iter().map(|node| (node.node_id.clone(), node.grpc_url.clone())).collect();
                        let stale: Vec<_> = watchers.iter()
                            .filter(|(id, (url, _))| live.get(*id) != Some(url))
                            .map(|(id, _)| id.clone())
                            .collect();
                        for id in stale {
                            if let Some((_, task)) = watchers.remove(&id) { task.abort(); }
                            projection.clear_for_node(&id, router.as_ref(), &registry, sessions.as_ref()).await;
                        }
                        for node in nodes {
                            if watchers.contains_key(&node.node_id) { continue; }
                            let gateway = match router.gateway_for_node(&node) {
                                Ok(gateway) => gateway,
                                Err(error) => {
                                    tracing::warn!(?error, node_id = %node.node_id, "invalid media node endpoint");
                                    continue;
                                }
                            };
                            let id = node.node_id.clone();
                            let url = node.grpc_url;
                            let task = tokio::spawn(watch_node(
                                projection.clone(), Arc::clone(&router), registry.clone(),
                                Arc::clone(&participants), Arc::clone(&sessions),
                                id.clone(), gateway, shutdown.clone(),
                            ));
                            watchers.insert(id, (url, task));
                        }
                    }
                }
            }
            for (_, (_, task)) in watchers {
                task.abort();
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
async fn watch_node(
    projection: SpeakerProjection,
    router: Arc<MediaPlacementRouter>,
    registry: ConnectionRegistry,
    participants: Arc<dyn ParticipantStore>,
    sessions: Arc<dyn SessionStore>,
    node_id: NodeId,
    media: Arc<GrpcMediaGateway>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        if *shutdown.borrow() {
            break;
        }
        let opened = tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { break; }
                continue;
            }
            result = tokio::time::timeout(Duration::from_secs(5), media.watch_active_speakers()) => result,
        };
        if let Ok(Ok(mut stream)) = opened {
            loop {
                tokio::select! {
                    changed = shutdown.changed() => {
                        if changed.is_err() || *shutdown.borrow() { return; }
                    }
                    item = stream.message() => match item {
                        Ok(Some(frame)) => projection.apply_from_node(
                            frame, &node_id, router.as_ref(), &registry,
                            participants.as_ref(), sessions.as_ref(),
                        ).await,
                        Ok(None) | Err(_) => break,
                    }
                }
            }
        }
        projection
            .clear_for_node(&node_id, router.as_ref(), &registry, sessions.as_ref())
            .await;
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() { break; }
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {}
        }
    }
}
