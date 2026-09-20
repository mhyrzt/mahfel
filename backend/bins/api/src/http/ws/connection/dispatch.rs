use super::{CLOSE_PROTOCOL_ERROR, writer::Connection};
use crate::http::ws::{
    envelope::{self, ClientMessage, ProtocolError, ServerMessage},
    metrics::DisconnectReason,
    registry::ConnectionRegistry,
};
use mahfel_application::ports::Principal;
use mahfel_domain::RoomId;
use mahfel_media_client::GrpcMediaGateway;
use uuid::Uuid;

fn parse_room_id(room_id: &str) -> Result<RoomId, String> {
    let uuid = Uuid::parse_str(room_id).map_err(|_| "invalid room_id".to_owned())?;
    Ok(RoomId::from_uuid(uuid))
}

pub(super) async fn handle_text(
    connection: &mut Connection,
    registry: &ConnectionRegistry,
    id: Uuid,
    text: &str,
    principal: &Principal,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let envelope = match envelope::parse_envelope(text) {
        Ok(env) => env,
        Err(ProtocolError::UnsupportedVersion(v)) => {
            tracing::debug!(%v, "unsupported protocol version");
            connection
                .close(CLOSE_PROTOCOL_ERROR, "unsupported protocol version")
                .await;
            return Err(DisconnectReason::ProtocolError);
        }
        Err(ProtocolError::Malformed) => {
            return connection
                .send(protocol_error("malformed", "malformed message"))
                .await;
        }
    };
    let rid = envelope.id.clone();
    let kind = rtc_kind(&envelope.message);

    match envelope.message {
        ClientMessage::Echo { text } => connection.send(ServerMessage::Echo { text }).await,
        ClientMessage::Broadcast { text } => {
            registry.broadcast(&ServerMessage::Broadcast {
                from: id.to_string(),
                text,
            });
            Ok(())
        }
        ClientMessage::Auth { .. } => Ok(()),
        ClientMessage::CreateTransport { room_id } => {
            handle_create_transport(connection, rid, kind, &room_id, principal, media).await
        }
        ClientMessage::ConnectTransport {
            room_id,
            transport_id,
            dtls_parameters,
        } => {
            handle_connect_transport(
                connection,
                rid,
                kind,
                room_id,
                transport_id,
                dtls_parameters,
                media,
            )
            .await
        }
        ClientMessage::Produce {
            room_id,
            transport_id,
            kind: produce_kind,
            rtp_parameters,
        } => {
            handle_produce(
                connection,
                rid,
                kind,
                room_id,
                transport_id,
                produce_kind,
                rtp_parameters,
                principal,
                media,
            )
            .await
        }
        ClientMessage::Consume {
            room_id,
            transport_id,
            producer_id,
            rtp_capabilities,
        } => {
            handle_consume(
                connection,
                rid,
                kind,
                room_id,
                transport_id,
                producer_id,
                rtp_capabilities,
                principal,
                media,
            )
            .await
        }
        ClientMessage::PauseConsumer {
            room_id,
            consumer_id,
        } => handle_pause(connection, rid, kind, room_id, consumer_id, media, false).await,
        ClientMessage::ResumeConsumer {
            room_id,
            consumer_id,
        } => handle_pause(connection, rid, kind, room_id, consumer_id, media, true).await,
    }
}

fn rtc_kind(msg: &ClientMessage) -> &'static str {
    match msg {
        ClientMessage::CreateTransport { .. } => "create_transport",
        ClientMessage::ConnectTransport { .. } => "connect_transport",
        ClientMessage::Produce { .. } => "produce",
        ClientMessage::Consume { .. } => "consume",
        ClientMessage::PauseConsumer { .. } => "pause_consumer",
        ClientMessage::ResumeConsumer { .. } => "resume_consumer",
        _ => "",
    }
}

async fn parse_room_or_send_error(
    connection: &mut Connection,
    rid: Option<String>,
    room_id: &str,
) -> Option<RoomId> {
    match parse_room_id(room_id) {
        Ok(r) => Some(r),
        Err(e) => {
            let _ = connection
                .send_with_id(rid, protocol_error("invalid_argument", &e))
                .await;
            None
        }
    }
}

async fn send_ok(
    connection: &mut Connection,
    rid: Option<String>,
    kind: &str,
    msg: ServerMessage,
) -> Result<(), DisconnectReason> {
    connection.send_with_id(rid, msg).await?;
    connection.metrics().rtc_request(kind, "success");
    Ok(())
}

async fn send_err(
    connection: &mut Connection,
    rid: Option<String>,
    kind: &str,
    status: tonic::Status,
) -> Result<(), DisconnectReason> {
    let (code, message) = map_status(status);
    connection
        .send_with_id(rid, protocol_error(&code, &message))
        .await?;
    connection.metrics().rtc_request(kind, "error");
    Ok(())
}

async fn handle_create_transport(
    c: &mut Connection,
    rid: Option<String>,
    kind: &str,
    room_id: &str,
    principal: &Principal,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let Some(room) = parse_room_or_send_error(c, rid.clone(), room_id).await else {
        return Ok(());
    };
    let pid = principal.user_id.to_string();
    match media.create_transport(room, pid).await {
        Ok((tid, ice_p, ice_c, dtls_p)) => {
            send_ok(
                c,
                rid,
                kind,
                ServerMessage::TransportCreated {
                    room_id: room.to_string(),
                    transport_id: tid,
                    ice_parameters: ice_p,
                    ice_candidates: ice_c,
                    dtls_parameters: dtls_p,
                },
            )
            .await
        }
        Err(s) => send_err(c, rid, kind, s).await,
    }
}

async fn handle_connect_transport(
    c: &mut Connection,
    rid: Option<String>,
    kind: &str,
    room_id: String,
    transport_id: String,
    dtls: String,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let Some(room) = parse_room_or_send_error(c, rid.clone(), &room_id).await else {
        return Ok(());
    };
    match media
        .connect_transport(room, transport_id.clone(), dtls)
        .await
    {
        Ok(()) => {
            send_ok(
                c,
                rid,
                kind,
                ServerMessage::TransportConnected {
                    room_id: room.to_string(),
                    transport_id,
                },
            )
            .await
        }
        Err(s) => send_err(c, rid, kind, s).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_produce(
    c: &mut Connection,
    rid: Option<String>,
    kind_label: &str,
    room_id: String,
    transport_id: String,
    produce_kind: String,
    rtp: String,
    principal: &Principal,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let Some(room) = parse_room_or_send_error(c, rid.clone(), &room_id).await else {
        return Ok(());
    };
    let pid = principal.user_id.to_string();
    match media
        .produce(room, pid, transport_id.clone(), produce_kind, rtp)
        .await
    {
        Ok(producer_id) => {
            send_ok(
                c,
                rid,
                kind_label,
                ServerMessage::Produced {
                    room_id: room.to_string(),
                    producer_id,
                },
            )
            .await
        }
        Err(s) => send_err(c, rid, kind_label, s).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_consume(
    c: &mut Connection,
    rid: Option<String>,
    kind: &str,
    room_id: String,
    transport_id: String,
    producer_id: String,
    rtp_cap: String,
    principal: &Principal,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let Some(room) = parse_room_or_send_error(c, rid.clone(), &room_id).await else {
        return Ok(());
    };
    let pid = principal.user_id.to_string();
    match media
        .consume(
            room,
            pid,
            producer_id.clone(),
            transport_id.clone(),
            rtp_cap,
        )
        .await
    {
        Ok((cid, rtp, paused)) => {
            send_ok(
                c,
                rid,
                kind,
                ServerMessage::Consumed {
                    room_id: room.to_string(),
                    consumer_id: cid,
                    rtp_parameters: rtp,
                    paused,
                },
            )
            .await
        }
        Err(s) => send_err(c, rid, kind, s).await,
    }
}

async fn handle_pause(
    c: &mut Connection,
    rid: Option<String>,
    kind: &str,
    room_id: String,
    consumer_id: String,
    media: &GrpcMediaGateway,
    resume: bool,
) -> Result<(), DisconnectReason> {
    let Some(room) = parse_room_or_send_error(c, rid.clone(), &room_id).await else {
        return Ok(());
    };
    let res = if resume {
        media.resume_consumer(room, consumer_id.clone()).await
    } else {
        media.pause_consumer(room, consumer_id.clone()).await
    };
    match res {
        Ok(()) => {
            let msg = if resume {
                ServerMessage::ConsumerResumed {
                    room_id: room.to_string(),
                    consumer_id,
                }
            } else {
                ServerMessage::ConsumerPaused {
                    room_id: room.to_string(),
                    consumer_id,
                }
            };
            send_ok(c, rid, kind, msg).await
        }
        Err(s) => send_err(c, rid, kind, s).await,
    }
}

fn map_status(s: tonic::Status) -> (String, String) {
    let c = match s.code() {
        tonic::Code::NotFound => "not_found",
        tonic::Code::InvalidArgument => "invalid_argument",
        tonic::Code::PermissionDenied => "permission_denied",
        tonic::Code::Unavailable => "unavailable",
        tonic::Code::DeadlineExceeded => "deadline_exceeded",
        _ => "internal",
    };
    (c.to_owned(), s.message().to_owned())
}

pub(super) fn protocol_error(code: &str, message: &str) -> ServerMessage {
    ServerMessage::Error {
        code: code.to_owned(),
        message: message.to_owned(),
    }
}
