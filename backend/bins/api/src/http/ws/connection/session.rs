use super::{
    CLOSE_GOING_AWAY, CLOSE_HEARTBEAT_TIMEOUT, SocketStream, dispatch, writer::Connection,
};
use crate::http::{
    auth::{AuthenticationError, RequestAuthenticator, principal_from_claims},
    ws::{
        envelope::{self, ClientMessage, ServerMessage},
        metrics::DisconnectReason,
        registry::{ConnectionRegistry, Outbound},
        state::{SESSION_PRESENCE_TTL, WsState},
    },
};
use axum::extract::ws::Message;
use futures_util::StreamExt;
use mahfel_application::ports::Principal;
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
    state: &WsState,
    mut shutdown: watch::Receiver<bool>,
    header_principal: Option<Principal>,
) -> DisconnectReason {
    // Deny by default: trust the upgrade header when present, otherwise require
    // an `auth` frame within the configured window.
    let principal = match header_principal {
        Some(principal) => principal,
        None => {
            let auth_timeout = connection.config().auth_timeout;
            match authenticate_first(
                &mut stream,
                authenticator,
                state.auth_tickets.as_ref(),
                auth_timeout,
            )
            .await
            {
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
    let registration = registry.register(id, principal.actor_id, tx);
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
        state,
    )
    .await;

    // A dropped socket still changes local presence. Keep the durable
    // participant Active so a reconnect reuses its identity, but close the
    // connected-time interval and notify the roster when the last device goes.
    if let Some(handle) = registry.participation(id) {
        let lifecycle_lock = registry.lifecycle_lock(handle.session_id, principal.actor_id);
        let lifecycle_guard = lifecycle_lock.lock().await;
        let global_guard = state
            .lock_session_actor(handle.session_id, principal.actor_id)
            .await;
        let remaining = match (&global_guard, &state.session_presence) {
            (Ok(_), Some(redis)) => redis
                .session_presence_leave(
                    handle.session_id,
                    id,
                    handle.participant_id,
                    SESSION_PRESENCE_TTL,
                )
                .await
                .map(Some),
            (Ok(_), None) => Ok(None),
            (Err(error), _) => {
                tracing::warn!(%error, "session lifecycle lock failed on disconnect");
                Ok(Some(u64::MAX))
            }
        };
        let remaining = match remaining {
            Ok(Some(count)) => count,
            Ok(None) => u64::try_from(
                registry
                    .connection_count_for_participant(handle.session_id, principal.actor_id)
                    .saturating_sub(1),
            )
            .unwrap_or(u64::MAX),
            Err(error) => {
                tracing::warn!(%error, "session presence leave failed on disconnect");
                u64::MAX
            }
        };
        if registry.left_if_current(id, handle) && remaining == 0 {
            if let Err(error) = state
                .stores
                .participation
                .close_open_interval(handle.participant_id, time::OffsetDateTime::now_utc())
                .await
            {
                tracing::warn!(
                    ?error,
                    participant_id = %handle.participant_id,
                    "failed to close participation interval on disconnect"
                );
            }
            registry.broadcast_to_session(
                handle.session_id,
                &ServerMessage::ParticipantLeft {
                    session_id: handle.session_id.to_string(),
                    participant_id: handle.participant_id.to_string(),
                },
            );
        }
        state.wake_session();
        drop(global_guard);
        drop(lifecycle_guard);
    }
    drop(registration);
    // This also covers EOF, heartbeat timeout, write failure, and shutdown.
    dispatch::close_owned_transports(connection, state).await;
    state.speakers.prune(registry);
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
    state: &WsState,
) -> Result<(), DisconnectReason> {
    let mut heartbeat = interval(connection.config().heartbeat_interval);
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
    // `interval` completes its first tick immediately; schedule the first ping
    // one full interval out so it cannot race the welcome frame.
    heartbeat.reset();
    let mut lease_heartbeat = interval(Duration::from_secs(10));
    lease_heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
    lease_heartbeat.reset();
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
                    dispatch::handle_text(connection, registry, id, text.as_str(), principal, state)
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
            _ = lease_heartbeat.tick() => {
                if let (Some(redis), Some(handle)) = (&state.session_presence, registry.participation(id)) {
                    match redis.session_presence_renew(
                        handle.session_id, id, handle.participant_id, SESSION_PRESENCE_TTL,
                    ).await {
                        Ok(true) => {}
                        Ok(false) => {
                            tracing::warn!("session presence lease expired; disconnecting socket");
                            return Err(DisconnectReason::Error);
                        }
                        Err(error) => {
                            tracing::warn!(%error, "session presence renewal failed");
                            return Err(DisconnectReason::Error);
                        }
                    }
                }
            }
        }
    }
}

/// Wait for the client's `auth` frame within `within`.
async fn authenticate_first(
    stream: &mut SocketStream,
    authenticator: &dyn RequestAuthenticator,
    ticket_store: Option<&mahfel_redis::RedisCoordination>,
    within: Duration,
) -> Result<Principal, DisconnectReason> {
    let attempt = async {
        loop {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => match envelope::parse(text.as_str()) {
                    Ok(ClientMessage::Auth { token, ticket }) => match (token, ticket) {
                        (Some(token), None) => {
                            let claims = authenticator.verify(Some(&token)).await?;
                            return principal_from_claims(&claims);
                        }
                        (None, Some(ticket)) => {
                            return super::super::tickets::consume(ticket_store, &ticket)
                                .await
                                .ok_or(AuthenticationError::MissingToken);
                        }
                        _ => return Err(AuthenticationError::MissingToken),
                    },
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
