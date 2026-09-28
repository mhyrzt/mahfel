use super::super::writer::Connection;
use crate::http::ws::{envelope::ServerMessage, metrics::DisconnectReason};
use mahfel_domain::RoomId;
use uuid::Uuid;

pub(super) fn parse_room_id(room_id: &str) -> Result<RoomId, String> {
    let uuid = Uuid::parse_str(room_id).map_err(|_| "invalid room_id".to_owned())?;
    Ok(RoomId::from_uuid(uuid))
}

pub(super) fn map_status(s: tonic::Status) -> (String, String) {
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

pub fn protocol_error(code: &str, message: &str) -> ServerMessage {
    ServerMessage::Error {
        code: code.to_owned(),
        message: message.to_owned(),
    }
}

pub(super) async fn parse_room_or_send_error(
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

pub(super) async fn send_ok(
    connection: &mut Connection,
    rid: Option<String>,
    kind: &str,
    msg: ServerMessage,
) -> Result<(), DisconnectReason> {
    connection.send_with_id(rid, msg).await?;
    connection.metrics().rtc_request(kind, "success");
    Ok(())
}

pub(super) async fn send_err(
    connection: &mut Connection,
    rid: Option<String>,
    kind: &str,
    status: tonic::Status,
) -> Result<(), DisconnectReason> {
    let detail = status.to_string();
    let (code, message) = map_status(status);
    tracing::warn!(operation = kind, code, detail, "rtc request failed");
    connection
        .send_with_id(rid, protocol_error(&code, &message))
        .await?;
    connection.metrics().rtc_request(kind, "error");
    Ok(())
}
