use super::SocketSink;
use crate::http::ws::{
    config::WsConfig,
    envelope::{self, ServerMessage},
    metrics::{DisconnectReason, WsMetrics},
};
use axum::extract::ws::{CloseFrame, Message};
use futures_util::SinkExt;
use mahfel_domain::{RoomId, SessionId};
use tokio::time::timeout;

/// Write half of a connection, bounded by the configured write timeout.
///
/// All writes go through here, so a stalled client always surfaces as a single
/// `Backpressure` outcome instead of a per-call-site timeout check.
pub(super) struct Connection {
    sink: SocketSink,
    config: WsConfig,
    metrics: WsMetrics,
    owned_transports: Vec<(SessionId, RoomId, String)>,
    owned_consumers: std::collections::HashSet<(RoomId, String)>,
    media: Option<ResolvedMedia>,
}

struct ResolvedMedia {
    session: SessionId,
    room: RoomId,
    expires: std::time::Instant,
    gateway: std::sync::Arc<mahfel_media_client::GrpcMediaGateway>,
}

impl Connection {
    #[must_use]
    pub(super) fn new(sink: SocketSink, config: WsConfig, metrics: WsMetrics) -> Self {
        Self {
            sink,
            config,
            metrics,
            owned_transports: Vec::new(),
            owned_consumers: std::collections::HashSet::new(),
            media: None,
        }
    }

    /// Connection limits, for the session's timers and queue sizing.
    pub(super) fn config(&self) -> &WsConfig {
        &self.config
    }

    /// Send one message. `Err(Backpressure)` means the write stalled or failed.
    pub(super) async fn send(&mut self, message: ServerMessage) -> Result<(), DisconnectReason> {
        let text = envelope::encode(message);
        let frame = Message::Text(text.into());
        match timeout(self.config.write_timeout, self.sink.send(frame)).await {
            Ok(Ok(())) => {
                self.metrics.message_sent();
                Ok(())
            }
            Ok(Err(_)) | Err(_) => Err(DisconnectReason::Backpressure),
        }
    }

    /// Send one message preserving the request id for correlation.
    pub(super) async fn send_with_id(
        &mut self,
        id: Option<String>,
        message: ServerMessage,
    ) -> Result<(), DisconnectReason> {
        let text = envelope::encode_with_id(id, message);
        let frame = Message::Text(text.into());
        match timeout(self.config.write_timeout, self.sink.send(frame)).await {
            Ok(Ok(())) => {
                self.metrics.message_sent();
                Ok(())
            }
            Ok(Err(_)) | Err(_) => Err(DisconnectReason::Backpressure),
        }
    }

    /// Send a heartbeat ping, bounded by the write timeout.
    pub(super) async fn ping(&mut self) -> Result<(), DisconnectReason> {
        let frame = Message::Ping(Vec::<u8>::new().into());
        match timeout(self.config.write_timeout, self.sink.send(frame)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) | Err(_) => Err(DisconnectReason::Backpressure),
        }
    }

    /// Best-effort close frame; the write timeout still applies.
    pub(super) async fn close(&mut self, code: u16, reason: &str) {
        let frame = CloseFrame {
            code,
            reason: reason.into(),
        };
        let _ = timeout(
            self.config.write_timeout,
            self.sink.send(Message::Close(Some(frame))),
        )
        .await;
    }

    /// Count one inbound message.
    pub(super) fn record_received(&self) {
        self.metrics.message_received();
    }

    pub(super) fn metrics(&self) -> &WsMetrics {
        &self.metrics
    }

    pub(super) fn own_transport(
        &mut self,
        session_id: SessionId,
        room_id: RoomId,
        transport_id: String,
    ) {
        self.owned_transports
            .push((session_id, room_id, transport_id));
    }

    pub(super) async fn media_for(
        &mut self,
        state: &crate::http::ws::state::WsState,
        session: SessionId,
        room: RoomId,
    ) -> Result<
        std::sync::Arc<mahfel_media_client::GrpcMediaGateway>,
        mahfel_application::placement::PlacementError,
    > {
        if let Some(cached) = &self.media
            && cached.session == session
            && cached.room == room
            && cached.expires > std::time::Instant::now()
        {
            return Ok(std::sync::Arc::clone(&cached.gateway));
        }
        self.media = None;
        let gateway = state.media_for(session, room).await?;
        self.media = Some(ResolvedMedia {
            session,
            room,
            expires: std::time::Instant::now() + std::time::Duration::from_secs(2),
            gateway: std::sync::Arc::clone(&gateway),
        });
        Ok(gateway)
    }

    pub(super) fn owns_transport(&self, room: RoomId, id: &str) -> bool {
        self.owned_transports
            .iter()
            .any(|(_, owned_room, owned_id)| *owned_room == room && owned_id == id)
    }

    pub(super) fn owns_consumer(&self, room: RoomId, id: &str) -> bool {
        self.owned_consumers.contains(&(room, id.to_owned()))
    }

    pub(super) fn own_consumer(&mut self, room: RoomId, id: String) {
        self.owned_consumers.insert((room, id));
    }

    pub(super) fn transport_count(&self) -> usize {
        self.owned_transports.len()
    }

    pub(super) fn take_transports(&mut self) -> Vec<(SessionId, RoomId, String)> {
        self.media = None;
        self.owned_consumers.clear();
        std::mem::take(&mut self.owned_transports)
    }
}
