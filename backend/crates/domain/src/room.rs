use crate::ids::entity_id;
use crate::status::status_conversion;
use crate::user::UserId;
use crate::validation::{RoomTitle, ValidationError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

entity_id!(RoomId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomStatus {
    Active,
    Archived,
}

/// Who may join a room.
///
/// `Public` rooms can be joined by anyone holding the link; `InviteOnly`
/// rooms are reachable only through an existing membership.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomVisibility {
    Public,
    InviteOnly,
}

/// Room-scoped role. Managers manage members and settings; members only
/// participate.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomRole {
    Manager,
    Member,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RoomError {
    #[error("room is already archived")]
    AlreadyArchived,
    #[error("invalid room title: {0}")]
    InvalidTitle(#[from] ValidationError),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub title: RoomTitle,
    pub created_by: UserId,
    pub visibility: RoomVisibility,
    pub status: RoomStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Room {
    #[must_use]
    pub fn create(
        title: RoomTitle,
        created_by: UserId,
        visibility: RoomVisibility,
        now: OffsetDateTime,
    ) -> Self {
        Self {
            id: RoomId::new(),
            title,
            created_by,
            visibility,
            status: RoomStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn rename(&mut self, title: RoomTitle, now: OffsetDateTime) -> Result<(), RoomError> {
        if self.status == RoomStatus::Archived {
            return Err(RoomError::AlreadyArchived);
        }
        self.title = title;
        self.updated_at = now;
        Ok(())
    }

    /// Change visibility. Existing memberships and their roles are preserved.
    pub fn set_visibility(
        &mut self,
        visibility: RoomVisibility,
        now: OffsetDateTime,
    ) -> Result<(), RoomError> {
        if self.status == RoomStatus::Archived {
            return Err(RoomError::AlreadyArchived);
        }
        self.visibility = visibility;
        self.updated_at = now;
        Ok(())
    }

    pub fn archive(&mut self, now: OffsetDateTime) -> Result<(), RoomError> {
        if self.status == RoomStatus::Archived {
            return Err(RoomError::AlreadyArchived);
        }
        self.status = RoomStatus::Archived;
        self.updated_at = now;
        Ok(())
    }
}

/// A user's membership in a room. Created automatically: adding a member or
/// joining a public room grants access immediately.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoomMember {
    pub room_id: RoomId,
    pub user_id: UserId,
    pub role: RoomRole,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl RoomMember {
    #[must_use]
    pub fn new(room_id: RoomId, user_id: UserId, role: RoomRole, now: OffsetDateTime) -> Self {
        Self {
            room_id,
            user_id,
            role,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn change_role(&mut self, role: RoomRole, now: OffsetDateTime) {
        self.role = role;
        self.updated_at = now;
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MembershipError {
    #[error("membership not found")]
    NotFound,
    #[error("the last manager cannot be removed")]
    LastManager,
}

/// Domain rule for removing a member: the target must exist and the last
/// manager may not be removed.
pub fn ensure_member_removable(
    members: &[RoomMember],
    target: UserId,
) -> Result<RoomRole, MembershipError> {
    let member = members
        .iter()
        .find(|member| member.user_id == target)
        .ok_or(MembershipError::NotFound)?;
    if member.role == RoomRole::Manager
        && members
            .iter()
            .filter(|member| member.role == RoomRole::Manager)
            .count()
            <= 1
    {
        return Err(MembershipError::LastManager);
    }
    Ok(member.role)
}

status_conversion!(RoomStatus, "room", { Active => "active", Archived => "archived" });
status_conversion!(RoomVisibility, "room visibility", { Public => "public", InviteOnly => "invite_only" });
status_conversion!(RoomRole, "room role", { Manager => "manager", Member => "member" });

#[cfg(test)]
mod tests {
    use super::*;

    fn room(visibility: RoomVisibility) -> Room {
        Room::create(
            RoomTitle::new("title".to_owned()).expect("valid"),
            UserId::new(),
            visibility,
            OffsetDateTime::now_utc(),
        )
    }

    #[test]
    fn status_storage_names_are_stable() {
        assert_eq!(RoomStatus::Active.as_str(), "active");
        assert_eq!(RoomStatus::Archived.as_str(), "archived");
        assert_eq!(RoomVisibility::InviteOnly.as_str(), "invite_only");
        assert_eq!(RoomRole::Manager.as_str(), "manager");
        assert_eq!(
            RoomRole::try_from("member").expect("known role"),
            RoomRole::Member
        );
        assert!(RoomRole::try_from("owner").is_err());
    }

    #[test]
    fn archive_is_terminal() {
        let now = OffsetDateTime::now_utc();
        let mut room = room(RoomVisibility::InviteOnly);
        assert_eq!(room.status, RoomStatus::Active);
        assert!(room.archive(now).is_ok());
        assert_eq!(room.status, RoomStatus::Archived);
        assert!(room.archive(now).is_err());
    }

    #[test]
    fn visibility_survives_transitions_until_archived() {
        let now = OffsetDateTime::now_utc();
        let mut room = room(RoomVisibility::InviteOnly);
        assert_eq!(room.visibility, RoomVisibility::InviteOnly);
        room.set_visibility(RoomVisibility::Public, now)
            .expect("active room");
        assert_eq!(room.visibility, RoomVisibility::Public);
        room.archive(now).expect("archive");
        assert!(
            room.set_visibility(RoomVisibility::InviteOnly, now)
                .is_err()
        );
    }

    #[test]
    fn last_manager_cannot_be_removed() {
        let now = OffsetDateTime::now_utc();
        let room_id = RoomId::new();
        let manager = UserId::new();
        let member = UserId::new();
        let mut members = vec![
            RoomMember::new(room_id, manager, RoomRole::Manager, now),
            RoomMember::new(room_id, member, RoomRole::Member, now),
        ];

        assert!(ensure_member_removable(&members, member).is_ok());
        assert_eq!(
            ensure_member_removable(&members, manager).expect_err("last manager"),
            MembershipError::LastManager
        );

        members.push(RoomMember::new(
            room_id,
            UserId::new(),
            RoomRole::Manager,
            now,
        ));
        assert!(ensure_member_removable(&members, manager).is_ok());
        assert_eq!(
            ensure_member_removable(&members, UserId::new()).expect_err("unknown"),
            MembershipError::NotFound
        );
    }
}
