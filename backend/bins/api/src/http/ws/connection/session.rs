use super::{
    CLOSE_GOING_AWAY, CLOSE_HEARTBEAT_TIMEOUT, SocketStream, dispatch, writer::Connection,
};
use crate::http::{
    auth::{AuthenticationError, RequestAuthenticator, principal_from_claims},
    ws::{
        envelope::{self, ClientMessage, ServerMessage},
        metrics::DisconnectReason,
        registry::{ConnectionRegistry, Outbound},
    },
};
use axum::extract::ws::Message;
use futures_util::StreamExt;
use mahfel_application::ports::Principal;
use mahfel_media_client::GrpcMediaGateway;
use std::time::{Duration, Instant};
use tokio::{
    sync::{mpsc, watch},
    time::{MissedTickBehavior, interval, timeout},
};
use uuid::Uuid;

/// Close code used when a connection is not authenticated in time.
pub(super) const CLOSE_UNAUTHORIZED: u16 = 4401;

/// Authenticate, register, and run the receive loop for one connection.
pub(super) async fn run(
    connection: &mut Connection,
    mut stream: SocketStream,
    registry: &ConnectionRegistry,
    authenticator: &dyn RequestAuthenticator,
    media: &GrpcMediaGateway,
    mut shutdown: watch::Receiver<bool>,
    header_principal: Option<Principal>,
) -> DisconnectReason {
    // Deny by default: trust the upgrade header when present, otherwise require
    // an `auth` frame within the configured window.
    let principal = match header_principal {
        Some(principal) => principal,
        None => {
            let auth_timeout = connection.config().auth_timeout;
            match authenticate_first(&mut stream, authenticator, auth_timeout).await {
                Ok(principal) => principal,
                Err(reason) => {
                    connection.close(CLOSE_UNAUTHORIZED, "unauthorized").await;
                    return reason;
                }
            }
        }
    };

    let id = Uuid::new_v4();
    let capacity = connection.config().outbound_capacity;
    let (tx, mut rx) = mpsc::channel::<Outbound>(capacity);
    let registration = registry.register(id, tx);
    tracing::debug!(user_id = %principal.actor_id, connection_id = %registration.id(), "websocket connected");

    let welcome = ServerMessage::Welcome {
        connection_id: registration.id().to_string(),
    };
    if let Err(reason) = connection.send(welcome).await {
        return reason;
    }

    let outcome = receive_loop(
        connection,
        &mut stream,
        registry,
        &mut shutdown,
        &mut rx,
        id,
        &principal,
        media,
    )
    .await;
    drop(registration);
    // The loop only ends by returning an error; an `Ok` completion is
    // unreachable and mapped defensively.
    outcome.err().unwrap_or(DisconnectReason::ClientClose)
}

/// The connection's select loop. `Err(reason)` means "disconnect for `reason`".
#[allow(clippy::too_many_arguments)]
async fn receive_loop(
    connection: &mut Connection,
    stream: &mut SocketStream,
    registry: &ConnectionRegistry,
    shutdown: &mut watch::Receiver<bool>,
    rx: &mut mpsc::Receiver<Outbound>,
    id: Uuid,
    principal: &Principal,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let mut heartbeat = interval(connection.config().heartbeat_interval);
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
    // `interval` completes its first tick immediately; schedule the first ping
    // one full interval out so it cannot race the welcome frame.
    heartbeat.reset();
    let heartbeat_timeout = connection.config().heartbeat_timeout;
    let mut last_seen = Instant::now();

    loop {
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    connection.close(CLOSE_GOING_AWAY, "shutdown").await;
                    return Err(DisconnectReason::Shutdown);
                }
            }
            outbound = rx.recv() => match outbound {
                None => return Err(DisconnectReason::Error),
                Some(Outbound::Message(message)) => connection.send(message).await?,
            },
            incoming = stream.next() => match incoming {
                None => return Err(DisconnectReason::ClientClose),
                Some(Err(_)) => return Err(DisconnectReason::Error),
                Some(Ok(Message::Close(_))) => return Err(DisconnectReason::ClientClose),
                Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {
                    last_seen = Instant::now();
                }
                Some(Ok(Message::Binary(_))) => {
                    let error = dispatch::protocol_error(
                        "binary_not_supported",
                        "binary frames are not supported",
                    );
                    connection.send(error).await?;
                }
                Some(Ok(Message::Text(text))) => {
                    last_seen = Instant::now();
                    connection.record_received();
                    dispatch::handle_text(connection, registry, id, text.as_str(), principal, media)
                        .await?;
                }
            },
            _ = heartbeat.tick() => {
                if last_seen.elapsed() >= heartbeat_timeout {
                    connection.close(CLOSE_HEARTBEAT_TIMEOUT, "heartbeat timeout").await;
                    return Err(DisconnectReason::HeartbeatTimeout);
                }
                connection.ping().await?;
            }
        }
    }
}

/// Wait for the client's `auth` frame within `within`.
async fn authenticate_first(
    stream: &mut SocketStream,
    authenticator: &dyn RequestAuthenticator,
    within: Duration,
) -> Result<Principal, DisconnectReason> {
    let attempt = async {
        loop {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => match envelope::parse(text.as_str()) {
                    Ok(ClientMessage::Auth { token }) => {
                        let claims = authenticator.verify(token.as_deref()).await?;
                        return principal_from_claims(&claims);
                    }
                    Ok(_) | Err(_) => return Err(AuthenticationError::MissingToken),
                },
                Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
                Some(Ok(Message::Close(_))) | None => {
                    return Err(AuthenticationError::MissingToken);
                }
                Some(Ok(Message::Binary(_))) => return Err(AuthenticationError::MissingToken),
                Some(Err(_)) => return Err(AuthenticationError::MissingToken),
            }
        }
    };

    match timeout(within, attempt).await {
        Ok(Ok(principal)) => Ok(principal),
        Ok(Err(_)) | Err(_) => Err(DisconnectReason::Unauthorized),
    }
}
