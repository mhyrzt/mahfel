use mahfel_domain::{AdmissionPolicy, Room, RoomMember, RoomRole, RoomVisibility};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Transport-level visibility schema. Kept out of the domain so `utoipa`
/// never reaches `mahfel-domain` (ADR 0015).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RoomVisibilityDto {
    Public,
    InviteOnly,
}

impl From<RoomVisibilityDto> for RoomVisibility {
    fn from(value: RoomVisibilityDto) -> Self {
        match value {
            RoomVisibilityDto::Public => Self::Public,
            RoomVisibilityDto::InviteOnly => Self::InviteOnly,
        }
    }
}

impl From<RoomVisibility> for RoomVisibilityDto {
    fn from(value: RoomVisibility) -> Self {
        match value {
            RoomVisibility::Public => Self::Public,
            RoomVisibility::InviteOnly => Self::InviteOnly,
        }
    }
}

/// Transport-level admission schema. Kept out of the domain so `utoipa` never
/// reaches `mahfel-domain` (ADR 0015).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AdmissionPolicyDto {
    Open,
    Lobby,
}

impl From<AdmissionPolicyDto> for AdmissionPolicy {
    fn from(value: AdmissionPolicyDto) -> Self {
        match value {
            AdmissionPolicyDto::Open => Self::Open,
            AdmissionPolicyDto::Lobby => Self::Lobby,
        }
    }
}

impl From<AdmissionPolicy> for AdmissionPolicyDto {
    fn from(value: AdmissionPolicy) -> Self {
        match value {
            AdmissionPolicy::Open => Self::Open,
            AdmissionPolicy::Lobby => Self::Lobby,
        }
    }
}

/// Transport-level role schema. Kept out of the domain so `utoipa` never
/// reaches `mahfel-domain` (ADR 0015).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RoomRoleDto {
    Manager,
    Member,
}

impl From<RoomRoleDto> for RoomRole {
    fn from(value: RoomRoleDto) -> Self {
        match value {
            RoomRoleDto::Manager => Self::Manager,
            RoomRoleDto::Member => Self::Member,
        }
    }
}

impl From<RoomRole> for RoomRoleDto {
    fn from(value: RoomRole) -> Self {
        match value {
            RoomRole::Manager => Self::Manager,
            RoomRole::Member => Self::Member,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateRoomRequest {
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) visibility: Option<RoomVisibilityDto>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateRoomRequest {
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) visibility: Option<RoomVisibilityDto>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct AddMemberRequest {
    /// Identifier (UUID) of the user to add.
    pub(crate) user_id: Uuid,
    #[serde(default)]
    pub(crate) role: Option<RoomRoleDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct RoomResponse {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) visibility: RoomVisibilityDto,
    pub(crate) admission: AdmissionPolicyDto,
    pub(crate) status: String,
    pub(crate) created_by: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct MemberResponse {
    pub(crate) user_id: String,
    pub(crate) role: RoomRoleDto,
}

impl From<Room> for RoomResponse {
    fn from(room: Room) -> Self {
        Self {
            id: room.id.to_string(),
            title: room.title.as_str().to_owned(),
            visibility: room.visibility.into(),
            admission: room.admission.into(),
            status: room.status.as_str().to_owned(),
            created_by: room.created_by.to_string(),
        }
    }
}

impl From<RoomMember> for MemberResponse {
    fn from(member: RoomMember) -> Self {
        Self {
            user_id: member.actor_id.to_string(),
            role: member.role.into(),
        }
    }
}
