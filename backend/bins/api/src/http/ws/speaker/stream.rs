//! Subscribe to the private media stream with bounded reconnects.

use super::SpeakerProjection;
use crate::http::ws::registry::ConnectionRegistry;
use mahfel_application::ports::{ParticipantStore, SessionStore};
use mahfel_media_client::GrpcMediaGateway;
use std::{sync::Arc, time::Duration};
use tokio::sync::watch;

impl SpeakerProjection {
    pub fn spawn(
        &self,
        media: Arc<GrpcMediaGateway>,
        registry: ConnectionRegistry,
        participants: Arc<dyn ParticipantStore>,
        sessions: Arc<dyn SessionStore>,
        mut shutdown: watch::Receiver<bool>,
    ) {
        let projection = self.clone();
        tokio::spawn(async move {
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
                match opened {
                    Err(_) => tracing::debug!("speaker stream connection timed out"),
                    Ok(Err(error)) => tracing::debug!(?error, "speaker stream unavailable"),
                    Ok(Ok(mut stream)) => loop {
                        tokio::select! {
                            changed = shutdown.changed() => {
                                if changed.is_err() || *shutdown.borrow() { return; }
                            }
                            item = stream.message() => match item {
                                Ok(Some(frame)) => projection.apply(frame, &registry, participants.as_ref(), sessions.as_ref()).await,
                                Ok(None) | Err(_) => break,
                            }
                        }
                    },
                }
                projection.clear(&registry);
                tokio::select! {
                    changed = shutdown.changed() => {
                        if changed.is_err() || *shutdown.borrow() { break; }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                }
            }
        });
    }
}
