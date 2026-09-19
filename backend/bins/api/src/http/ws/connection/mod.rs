//! Per-connection WebSocket lifecycle.
//!
//! [`run`] is the entry point handed to `WebSocketUpgrade::on_upgrade`; it
//! splits the socket, delegates the session to [`session`], and records the
//! disconnect outcome once. The writer half lives in [`writer`] and client
//! frame dispatch in [`dispatch`].

mod dispatch;
mod session;
mod writer;

use super::registry::ConnectionGuard;
use super::state::WsState;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    StreamExt,
    stream::{SplitSink, SplitStream},
};
use mahfel_application::ports::Principal;
use writer::Connection;

/// Writer half of a split WebSocket.
pub(super) type SocketSink = SplitSink<WebSocket, Message>;
/// Reader half of a split WebSocket.
pub(super) type SocketStream = SplitStream<WebSocket>;

/// Close code used when a connection outlives its heartbeat window.
pub(super) const CLOSE_HEARTBEAT_TIMEOUT: u16 = 4408;
/// Close code used on protocol errors (RFC 6455 reserved range).
pub(super) const CLOSE_PROTOCOL_ERROR: u16 = 1002;
/// Close code used for a graceful server shutdown.
pub(super) const CLOSE_GOING_AWAY: u16 = 1001;

/// Drive one WebSocket connection from authentication to close.
///
/// `_guard` holds the capacity reservation for this connection's lifetime.
pub(super) async fn run(
    socket: WebSocket,
    state: WsState,
    _guard: ConnectionGuard,
    header_principal: Option<Principal>,
) {
    let WsState {
        registry,
        config,
        metrics,
        authenticator,
        shutdown,
    } = state;
    let (sink, stream) = socket.split();
    let mut connection = Connection::new(sink, config, metrics.clone());

    // The session reports why it ended; recording the disconnect in one place
    // keeps every exit path (auth, protocol, idle, shutdown, I/O) consistent.
    let reason = session::run(
        &mut connection,
        stream,
        &registry,
        authenticator.as_ref(),
        shutdown,
        header_principal,
    )
    .await;
    metrics.disconnected(reason);
}
