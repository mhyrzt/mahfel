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

/// Apply one client text frame. `Err(reason)` requests a disconnect.
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
        Err(ProtocolError::UnsupportedVersion(version)) => {
            tracing::debug!(%version, "websocket closed: unsupported protocol version");
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
    let request_id = envelope.id.clone();
    let metrics_kind = match &envelope.message {
        ClientMessage::CreateTransport { .. } => "create_transport",
        ClientMessage::ConnectTransport { .. } => "connect_transport",
        ClientMessage::Produce { .. } => "produce",
        ClientMessage::Consume { .. } => "consume",
        ClientMessage::PauseConsumer { .. } => "pause_consumer",
        ClientMessage::ResumeConsumer { .. } => "resume_consumer",
        _ => "",
    };

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
            let room = match parse_room_id(&room_id) {
                Ok(r) => r,
                Err(e) => {
                    connection
                        .send_with_id(request_id, protocol_error("invalid_argument", &e))
                        .await?;
                    return Ok(());
                }
            };
            let participant_id = principal.user_id.to_string();
            match media.create_transport(room, participant_id).await {
                Ok((transport_id, ice_parameters, ice_candidates, dtls_parameters)) => {
                    connection
                        .send_with_id(
                            request_id.clone(),
                            ServerMessage::TransportCreated {
                                room_id: room.to_string(),
                                transport_id,
                                ice_parameters,
                                ice_candidates,
                                dtls_parameters,
                            },
                        )
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "success");
                    Ok(())
                }
                Err(status) => {
                    let (code, message) = map_status(status);
                    connection
                        .send_with_id(request_id.clone(), protocol_error(&code, &message))
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "error");
                    Ok(())
                }
            }
        }
        ClientMessage::ConnectTransport {
            room_id,
            transport_id,
            dtls_parameters,
        } => {
            let room = match parse_room_id(&room_id) {
                Ok(r) => r,
                Err(e) => {
                    connection
                        .send_with_id(request_id, protocol_error("invalid_argument", &e))
                        .await?;
                    return Ok(());
                }
            };
            match media
                .connect_transport(room, transport_id.clone(), dtls_parameters)
                .await
            {
                Ok(()) => {
                    connection
                        .send_with_id(
                            request_id.clone(),
                            ServerMessage::TransportConnected {
                                room_id: room.to_string(),
                                transport_id,
                            },
                        )
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "success");
                    Ok(())
                }
                Err(status) => {
                    let (code, message) = map_status(status);
                    connection
                        .send_with_id(request_id.clone(), protocol_error(&code, &message))
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "error");
                    Ok(())
                }
            }
        }
        ClientMessage::Produce {
            room_id,
            transport_id,
            kind,
            rtp_parameters,
        } => {
            let room = match parse_room_id(&room_id) {
                Ok(r) => r,
                Err(e) => {
                    connection
                        .send_with_id(request_id, protocol_error("invalid_argument", &e))
                        .await?;
                    return Ok(());
                }
            };
            let participant_id = principal.user_id.to_string();
            match media
                .produce(
                    room,
                    participant_id,
                    transport_id.clone(),
                    kind,
                    rtp_parameters,
                )
                .await
            {
                Ok(producer_id) => {
                    connection
                        .send_with_id(
                            request_id.clone(),
                            ServerMessage::Produced {
                                room_id: room.to_string(),
                                producer_id,
                            },
                        )
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "success");
                    Ok(())
                }
                Err(status) => {
                    let (code, message) = map_status(status);
                    connection
                        .send_with_id(request_id.clone(), protocol_error(&code, &message))
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "error");
                    Ok(())
                }
            }
        }
        ClientMessage::Consume {
            room_id,
            transport_id,
            producer_id,
            rtp_capabilities,
        } => {
            let room = match parse_room_id(&room_id) {
                Ok(r) => r,
                Err(e) => {
                    connection
                        .send_with_id(request_id, protocol_error("invalid_argument", &e))
                        .await?;
                    return Ok(());
                }
            };
            let participant_id = principal.user_id.to_string();
            match media
                .consume(
                    room,
                    participant_id,
                    producer_id.clone(),
                    transport_id.clone(),
                    rtp_capabilities,
                )
                .await
            {
                Ok((consumer_id, rtp_parameters, paused)) => {
                    connection
                        .send_with_id(
                            request_id.clone(),
                            ServerMessage::Consumed {
                                room_id: room.to_string(),
                                consumer_id,
                                rtp_parameters,
                                paused,
                            },
                        )
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "success");
                    Ok(())
                }
                Err(status) => {
                    let (code, message) = map_status(status);
                    connection
                        .send_with_id(request_id.clone(), protocol_error(&code, &message))
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "error");
                    Ok(())
                }
            }
        }
        ClientMessage::PauseConsumer {
            room_id,
            consumer_id,
        } => {
            let room = match parse_room_id(&room_id) {
                Ok(r) => r,
                Err(e) => {
                    connection
                        .send_with_id(request_id, protocol_error("invalid_argument", &e))
                        .await?;
                    return Ok(());
                }
            };
            match media.pause_consumer(room, consumer_id.clone()).await {
                Ok(()) => {
                    connection
                        .send_with_id(
                            request_id.clone(),
                            ServerMessage::ConsumerPaused {
                                room_id: room.to_string(),
                                consumer_id,
                            },
                        )
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "success");
                    Ok(())
                }
                Err(status) => {
                    let (code, message) = map_status(status);
                    connection
                        .send_with_id(request_id.clone(), protocol_error(&code, &message))
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "error");
                    Ok(())
                }
            }
        }
        ClientMessage::ResumeConsumer {
            room_id,
            consumer_id,
        } => {
            let room = match parse_room_id(&room_id) {
                Ok(r) => r,
                Err(e) => {
                    connection
                        .send_with_id(request_id, protocol_error("invalid_argument", &e))
                        .await?;
                    return Ok(());
                }
            };
            match media.resume_consumer(room, consumer_id.clone()).await {
                Ok(()) => {
                    connection
                        .send_with_id(
                            request_id.clone(),
                            ServerMessage::ConsumerResumed {
                                room_id: room.to_string(),
                                consumer_id,
                            },
                        )
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "success");
                    Ok(())
                }
                Err(status) => {
                    let (code, message) = map_status(status);
                    connection
                        .send_with_id(request_id.clone(), protocol_error(&code, &message))
                        .await?;
                    connection.metrics().rtc_request(metrics_kind, "error");
                    Ok(())
                }
            }
        }
    }
}

fn map_status(status: tonic::Status) -> (String, String) {
    let code = match status.code() {
        tonic::Code::NotFound => "not_found",
        tonic::Code::InvalidArgument => "invalid_argument",
        tonic::Code::PermissionDenied => "permission_denied",
        tonic::Code::Unavailable => "unavailable",
        tonic::Code::DeadlineExceeded => "deadline_exceeded",
        _ => "internal",
    };
    (code.to_owned(), status.message().to_owned())
}

/// Build a server error frame.
pub(super) fn protocol_error(code: &str, message: &str) -> ServerMessage {
    ServerMessage::Error {
        code: code.to_owned(),
        message: message.to_owned(),
    }
}
