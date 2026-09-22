use crate::actor::ActorId;
use crate::ids::entity_id;
use crate::room::RoomId;
use crate::session::SessionId;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

entity_id!(BlockId);

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BlockError {
    #[error("block is already lifted")]
    AlreadyLifted,
    #[error("an expiry must be after the block was imposed")]
    ExpiryBeforeBlock,
}

/// What a block bars a principal from.
///
/// The two are independent durable facts rather than one being derived from the
/// other. A room block denies every session in its room by enforcement, so no
/// session rows are written for it, and lifting it can therefore never disturb
/// an unrelated session block.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockScope {
    Room(RoomId),
    Session(SessionId),
}

impl BlockScope {
    #[must_use]
    pub const fn is_room(self) -> bool {
        matches!(self, Self::Room(_))
    }

    /// The room the block is authorized against.
    ///
    /// A session block resolves through its session, so this is only the room
    /// for a room block.
    #[must_use]
    pub const fn room_id(self) -> Option<RoomId> {
        match self {
            Self::Room(room_id) => Some(room_id),
            Self::Session(_) => None,
        }
    }

    #[must_use]
    pub const fn session_id(self) -> Option<SessionId> {
        match self {
            Self::Room(_) => None,
            Self::Session(session_id) => Some(session_id),
        }
    }
}

/// A durable bar on a principal rejoining a room or a session.
///
/// Unlike a mute, a block outlives the participation it was imposed during:
/// that is its purpose. `expires_at` is optional because the two settings
/// disagree: an education deployment wants "until I lift it", while a one-off
/// meeting wants a block that lapses on its own.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub scope: BlockScope,
    pub actor_id: ActorId,
    pub blocked_by: ActorId,
    pub reason: Option<String>,
    pub blocked_at: OffsetDateTime,
    pub expires_at: Option<OffsetDateTime>,
    pub lifted_at: Option<OffsetDateTime>,
    pub lifted_by: Option<ActorId>,
}

impl Block {
    /// Impose a block.
    ///
    /// # Errors
    /// [`BlockError::ExpiryBeforeBlock`] when an expiry is not after the
    /// imposition, which would create a block that was never active.
    pub fn new(
        scope: BlockScope,
        actor_id: ActorId,
        blocked_by: ActorId,
        reason: Option<String>,
        expires_at: Option<OffsetDateTime>,
        now: OffsetDateTime,
    ) -> Result<Self, BlockError> {
        if expires_at.is_some_and(|expires| expires <= now) {
            return Err(BlockError::ExpiryBeforeBlock);
        }
        Ok(Self {
            id: BlockId::new(),
            scope,
            actor_id,
            blocked_by,
            reason,
            blocked_at: now,
            expires_at,
            lifted_at: None,
            lifted_by: None,
        })
    }

    /// Whether the block still bars a join at `now`.
    ///
    /// A block that has expired is inactive without being lifted: it was never
    /// explicitly cleared, it simply ran out.
    #[must_use]
    pub fn is_active_at(&self, now: OffsetDateTime) -> bool {
        self.lifted_at.is_none() && self.expires_at.is_none_or(|expires| expires > now)
    }

    /// Lift the block before it expires.
    ///
    /// # Errors
    /// [`BlockError::AlreadyLifted`] on a repeat, so a lift cannot overwrite
    /// who lifted it or when.
    pub fn lift(&mut self, by: ActorId, now: OffsetDateTime) -> Result<(), BlockError> {
        if self.lifted_at.is_some() {
            return Err(BlockError::AlreadyLifted);
        }
        self.lifted_at = Some(now);
        self.lifted_by = Some(by);
        Ok(())
    }

    /// Extend an active block to a new expiry.
    ///
    /// Re-blocking an already-blocked principal extends rather than inserting a
    /// second row, so there is never more than one active block to resolve.
    ///
    /// # Errors
    /// [`BlockError::ExpiryBeforeBlock`] when the new expiry is not after the
    /// imposition.
    pub fn extend(&mut self, expires_at: Option<OffsetDateTime>) -> Result<(), BlockError> {
        if expires_at.is_some_and(|expires| expires <= self.blocked_at) {
            return Err(BlockError::ExpiryBeforeBlock);
        }
        self.expires_at = expires_at;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(seconds: i64) -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(seconds)
    }

    fn block(expires_at: Option<OffsetDateTime>) -> Block {
        Block::new(
            BlockScope::Room(RoomId::new()),
            ActorId::new(),
            ActorId::new(),
            None,
            expires_at,
            at(0),
        )
        .expect("block")
    }

    #[test]
    fn a_scope_carries_exactly_one_target() {
        let room = RoomId::new();
        let session = SessionId::new();

        let room_scope = BlockScope::Room(room);
        assert!(room_scope.is_room());
        assert_eq!(room_scope.room_id(), Some(room));
        assert!(room_scope.session_id().is_none());

        let session_scope = BlockScope::Session(session);
        assert!(!session_scope.is_room());
        assert_eq!(session_scope.session_id(), Some(session));
        assert!(session_scope.room_id().is_none());
    }

    #[test]
    fn a_new_block_is_active_and_unlifted() {
        let block = block(None);
        assert!(block.is_active_at(at(0)));
        assert!(block.is_active_at(at(1_000_000)));
        assert!(block.lifted_at.is_none());
    }

    #[test]
    fn a_block_without_an_expiry_holds_until_lifted() {
        let mut block = block(None);
        assert!(block.is_active_at(at(100)));
        block.lift(ActorId::new(), at(200)).expect("lift");
        assert!(
            !block.is_active_at(at(300)),
            "an explicit lift ends the block"
        );
    }

    #[test]
    fn a_block_with_an_expiry_lapses_on_its_own() {
        let block = block(Some(at(100)));
        assert!(
            block.is_active_at(at(50)),
            "an unexpired block is still active"
        );
        assert!(
            !block.is_active_at(at(100)),
            "the block ends at its expiry, not after it"
        );
        assert!(
            !block.is_active_at(at(200)),
            "it lapses without being lifted"
        );
        assert!(block.lifted_at.is_none(), "lapsing is not a lift");
    }

    #[test]
    fn lifting_twice_is_refused() {
        let mut block = block(None);
        block.lift(ActorId::new(), at(10)).expect("lift");
        assert_eq!(
            block.lift(ActorId::new(), at(20)),
            Err(BlockError::AlreadyLifted),
            "a repeat lift must not overwrite who lifted it"
        );
        assert_eq!(block.lifted_at, Some(at(10)));
    }

    #[test]
    fn extending_replaces_the_expiry_rather_than_stacking() {
        let mut block = block(Some(at(100)));
        block.extend(Some(at(500))).expect("extend");
        assert_eq!(block.expires_at, Some(at(500)));
        assert!(
            block.is_active_at(at(400)),
            "the extension is what takes effect"
        );
    }

    #[test]
    fn a_block_can_be_made_indefinite_by_extending_without_an_expiry() {
        let mut block = block(Some(at(100)));
        block.extend(None).expect("extend");
        assert!(block.is_active_at(at(1_000_000)));
    }

    #[test]
    fn an_expiry_before_the_block_is_refused() {
        assert_eq!(
            Block::new(
                BlockScope::Room(RoomId::new()),
                ActorId::new(),
                ActorId::new(),
                None,
                Some(at(-1)),
                at(0),
            ),
            Err(BlockError::ExpiryBeforeBlock)
        );
    }

    #[test]
    fn extending_to_before_the_block_is_refused() {
        let mut block = block(None);
        assert_eq!(
            block.extend(Some(at(-1))),
            Err(BlockError::ExpiryBeforeBlock)
        );
    }
}
