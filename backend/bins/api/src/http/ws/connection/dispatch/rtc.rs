use super::super::writer::Connection;
use super::helpers::{parse_room_or_send_error, send_err, send_ok};
use crate::http::ws::{envelope::ServerMessage, metrics::DisconnectReason};
use mahfel_application::ports::Principal;
use mahfel_media_client::GrpcMediaGateway;
use mediasoup::prelude::MediaKind;

pub(super) async fn handle_create_transport(
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
    match media.create_transport(room, principal.actor_id).await {
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

pub(super) async fn handle_connect_transport(
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
pub(super) async fn handle_produce(
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
    let kind = match produce_kind.as_str() {
        "audio" => MediaKind::Audio,
        "video" => MediaKind::Video,
        _ => {
            let _ = c
                .send_with_id(
                    rid.clone(),
                    super::helpers::protocol_error("invalid_argument", "invalid kind"),
                )
                .await;
            return Ok(());
        }
    };
    match media
        .produce(room, principal.actor_id, transport_id.clone(), kind, rtp)
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
pub(super) async fn handle_consume(
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
    match media
        .consume(
            room,
            principal.actor_id,
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

pub(super) async fn handle_pause(
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
