use mahfel_domain::{RoomId, UserId};
use mediasoup::prelude::MediaKind;
use tonic::Status;
use uuid::Uuid;

#[inline]
pub(crate) fn saturating_u64(v: usize) -> u64 {
    u64::try_from(v).unwrap_or(u64::MAX)
}

#[inline]
pub(crate) fn saturating_u32(v: usize) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

#[inline]
pub(crate) fn saturating_i64(v: usize) -> i64 {
    i64::try_from(v).unwrap_or(0)
}

pub(crate) fn require_field(raw: String, field: &str) -> Result<String, Status> {
    let trimmed = raw.trim().to_owned();
    if trimmed.is_empty() {
        return Err(Status::invalid_argument(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed)
}

pub(crate) fn parse_room_id(raw: &str) -> Result<RoomId, Status> {
    let uuid = Uuid::parse_str(raw).map_err(|_| Status::invalid_argument("invalid room_id"))?;
    Ok(RoomId::from_uuid(uuid))
}

pub(crate) fn parse_user_id(raw: &str) -> Result<UserId, Status> {
    let uuid =
        Uuid::parse_str(raw).map_err(|_| Status::invalid_argument("invalid participant_id"))?;
    Ok(UserId::from_uuid(uuid))
}

pub(crate) fn parse_media_kind(raw: &str) -> Result<MediaKind, Status> {
    match raw {
        "audio" => Ok(MediaKind::Audio),
        "video" => Ok(MediaKind::Video),
        _ => Err(Status::invalid_argument("invalid kind")),
    }
}
