use super::SocketSink;
use crate::http::ws::{
    config::WsConfig,
    envelope::{self, ServerMessage},
    metrics::{DisconnectReason, WsMetrics},
};
use axum::extract::ws::{CloseFrame, Message};
use futures_util::SinkExt;
use tokio::time::timeout;

/// Write half of a connection, bounded by the configured write timeout.
///
/// All writes go through here, so a stalled client always surfaces as a single
/// `Backpressure` outcome instead of a per-call-site timeout check.
pub(super) struct Connection {
    sink: SocketSink,
    config: WsConfig,
    metrics: WsMetrics,
}

impl Connection {
    #[must_use]
    pub(super) fn new(sink: SocketSink, config: WsConfig, metrics: WsMetrics) -> Self {
        Self {
            sink,
            config,
            metrics,
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
}
