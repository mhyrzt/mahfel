use mahfel_domain::{Room, RoomMember, RoomRole, RoomVisibility};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub(super) struct CreateRoomRequest {
    pub(super) title: String,
    #[serde(default)]
    pub(super) visibility: Option<RoomVisibility>,
}

#[derive(Debug, Deserialize)]
pub(super) struct UpdateRoomRequest {
    #[serde(default)]
    pub(super) title: Option<String>,
    #[serde(default)]
    pub(super) visibility: Option<RoomVisibility>,
}

#[derive(Debug, Deserialize)]
pub(super) struct AddMemberRequest {
    pub(super) user_id: Uuid,
    #[serde(default)]
    pub(super) role: Option<RoomRole>,
}

#[derive(Debug, Serialize)]
pub(super) struct RoomResponse {
    id: String,
    title: String,
    visibility: RoomVisibility,
    status: String,
    created_by: String,
}

#[derive(Debug, Serialize)]
pub(super) struct MemberResponse {
    user_id: String,
    role: RoomRole,
}

impl From<Room> for RoomResponse {
    fn from(room: Room) -> Self {
        Self {
            id: room.id.to_string(),
            title: room.title.as_str().to_owned(),
            visibility: room.visibility,
            status: room.status.as_str().to_owned(),
            created_by: room.created_by.to_string(),
        }
    }
}

impl From<RoomMember> for MemberResponse {
    fn from(member: RoomMember) -> Self {
        Self {
            user_id: member.user_id.to_string(),
            role: member.role,
        }
    }
}
