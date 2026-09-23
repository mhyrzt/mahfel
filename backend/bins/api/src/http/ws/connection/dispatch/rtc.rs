use super::super::writer::Connection;
use super::helpers::{parse_room_or_send_error, send_err, send_ok};
use crate::http::ws::{envelope::ServerMessage, metrics::DisconnectReason, state::WsState};
use mahfel_application::ports::{Capability, Principal};
use mahfel_domain::{MediaKind, ProducerSource, RoomId, SessionId};
use mahfel_media_client::GrpcMediaGateway;

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_create_transport(
    c: &mut Connection,
    rid: Option<String>,
    kind: &str,
    room_id: &str,
    principal: &Principal,
    session_id: Option<SessionId>,
    media_room: Option<RoomId>,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let Some(room) = parse_room_or_send_error(c, rid.clone(), room_id).await else {
        return Ok(());
    };
    match media
        .create_transport(media_room.unwrap_or(room), principal.actor_id)
        .await
    {
        Ok((tid, ice_p, ice_c, dtls_p)) => {
            if let Some(session_id) = session_id {
                c.own_transport(session_id, room, tid.clone());
            }
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

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_connect_transport(
    c: &mut Connection,
    rid: Option<String>,
    kind: &str,
    room_id: String,
    transport_id: String,
    dtls: String,
    media_room: Option<RoomId>,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let Some(room) = parse_room_or_send_error(c, rid.clone(), &room_id).await else {
        return Ok(());
    };
    match media
        .connect_transport(media_room.unwrap_or(room), transport_id.clone(), dtls)
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

/// Resolve a requested source, defaulting to the kind's obvious meaning.
fn parse_source(raw: Option<&str>, kind: MediaKind) -> Result<ProducerSource, &'static str> {
    let Some(raw) = raw.map(str::trim).filter(|raw| !raw.is_empty()) else {
        return Ok(match kind {
            MediaKind::Audio => ProducerSource::Microphone,
            MediaKind::Video => ProducerSource::Camera,
        });
    };
    let source: ProducerSource = raw.try_into().map_err(|_| "invalid source")?;
    if source.is_audio() != (kind == MediaKind::Audio) {
        return Err("source does not match kind");
    }
    Ok(source)
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
    source: Option<String>,
    principal: &Principal,
    state: &WsState,
    media_room: Option<RoomId>,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let Some(room) = parse_room_or_send_error(c, rid.clone(), &room_id).await else {
        return Ok(());
    };
    let kind = match MediaKind::try_from(produce_kind.as_str()) {
        Ok(kind) => kind,
        Err(_) => {
            let _ = c
                .send_with_id(
                    rid.clone(),
                    super::helpers::protocol_error("invalid_argument", "invalid kind"),
                )
                .await;
            return Ok(());
        }
    };
    // An omitted source takes the kind's obvious meaning, so a client that
    // predates the field keeps working. A mismatched source is refused rather
    // than coerced: it selects which producers a moderation action applies to,
    // so a wrong label would misdirect a later mute.
    let source = match parse_source(source.as_deref(), kind) {
        Ok(source) => source,
        Err(message) => {
            let _ = c
                .send_with_id(
                    rid.clone(),
                    super::helpers::protocol_error("invalid_argument", message),
                )
                .await;
            return Ok(());
        }
    };

    // Publishing is capability-gated by source, because ShareScreen is a
    // distinct capability from PublishMedia: a room can permit speaking without
    // permitting a screen share, and a non-member in a public room holds
    // neither. Checked before any media is created, so an unauthorized caller
    // never reaches the SFU.
    let capability = match source {
        ProducerSource::Screen => Capability::ShareScreen,
        ProducerSource::Microphone | ProducerSource::Camera => Capability::PublishMedia,
    };
    if mahfel_application::policy::require_room_capability(
        state.stores.rooms.as_ref(),
        principal,
        room,
        capability,
    )
    .await
    .is_err()
    {
        let _ = c
            .send_with_id(
                rid,
                super::helpers::protocol_error("forbidden", "publishing is not permitted"),
            )
            .await;
        return Ok(());
    }
    match media
        .produce(
            media_room.unwrap_or(room),
            principal.actor_id,
            transport_id.clone(),
            kind,
            source,
            rtp,
        )
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
    media_room: Option<RoomId>,
    media: &GrpcMediaGateway,
) -> Result<(), DisconnectReason> {
    let Some(room) = parse_room_or_send_error(c, rid.clone(), &room_id).await else {
        return Ok(());
    };
    match media
        .consume(
            media_room.unwrap_or(room),
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

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_pause(
    c: &mut Connection,
    rid: Option<String>,
    kind: &str,
    room_id: String,
    consumer_id: String,
    media_room: Option<RoomId>,
    media: &GrpcMediaGateway,
    resume: bool,
) -> Result<(), DisconnectReason> {
    let Some(room) = parse_room_or_send_error(c, rid.clone(), &room_id).await else {
        return Ok(());
    };
    let res = if resume {
        media
            .resume_consumer(media_room.unwrap_or(room), consumer_id.clone())
            .await
    } else {
        media
            .pause_consumer(media_room.unwrap_or(room), consumer_id.clone())
            .await
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
