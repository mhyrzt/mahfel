use super::{BoxFuture, StoreError};
use mahfel_domain::{
    ActorId, ChatChannel, ChatEvent, ChatMessage, ChatMessageCursor, ChatMessageDraft,
    ChatMessageId, ChatMessagePage, RoomId, SessionId,
};
use time::OffsetDateTime;

/// Aggregated emoji counts and the caller's own choice.
pub type ChatReactionSnapshot = (Vec<(String, i64)>, Option<String>);

/// Persistence for room-scoped chat channels and their durable messages.
pub trait ChatStore: Send + Sync {
    /// Current members authorized for fanout; never cache across delivery batches.
    fn authorized_subscribers<'a>(
        &'a self,
        room_id: RoomId,
        actors: &'a [ActorId],
    ) -> BoxFuture<'a, Result<Vec<ActorId>, StoreError>>;

    /// Private per-actor cursor, checked against current room membership.
    fn read_marker(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<i64, StoreError>>;

    /// Move the private cursor forward, never backward or past the event head.
    fn advance_read_marker(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        sequence: i64,
        now: OffsetDateTime,
    ) -> BoxFuture<'_, Result<i64, StoreError>>;

    /// Current committed sequence for a room, including zero before its first
    /// message. Used to reject impossible resume cursors.
    fn event_head(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
    ) -> BoxFuture<'_, Result<i64, StoreError>>;

    /// Read committed changes after an exclusive channel sequence. Membership
    /// is checked on every call so a revoked subscriber cannot keep receiving
    /// room content. The result is ascending and capped by `limit`.
    fn list_events(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        after_sequence: i64,
        limit: u32,
    ) -> BoxFuture<'_, Result<Vec<ChatEvent>, StoreError>>;

    fn list_replies<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
        parent_id: ChatMessageId,
        before: Option<&'a ChatMessageCursor>,
        limit: u32,
    ) -> BoxFuture<'a, Result<ChatMessagePage, StoreError>>;

    fn list_mentions(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
    ) -> BoxFuture<'_, Result<Vec<ActorId>, StoreError>>;

    fn set_reaction<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
        emoji: &'a str,
        now: OffsetDateTime,
    ) -> BoxFuture<'a, Result<(), StoreError>>;

    fn remove_reaction(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
        now: OffsetDateTime,
    ) -> BoxFuture<'_, Result<(), StoreError>>;

    fn list_reactions(
        &self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
    ) -> BoxFuture<'_, Result<ChatReactionSnapshot, StoreError>>;

    /// Persist a message only if its author is a current member of the channel's room.
    ///
    /// The adapter enforces membership in the same transaction as the write so a
    /// concurrent removal cannot race message creation. Unauthorized or missing
    /// rooms are reported as [`StoreError::NotFound`] to avoid resource probing.
    fn create_message<'a>(
        &'a self,
        channel: &'a ChatChannel,
        draft: &'a ChatMessageDraft,
    ) -> BoxFuture<'a, Result<ChatMessage, StoreError>>;

    /// List newest-first messages visible to a current room member. The cursor
    /// is an exclusive `(created_at, id)` keyset boundary. `session_id` selects
    /// a session conversation; `None` selects messages with no session scope.
    fn list_messages<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
        session_id: Option<SessionId>,
        before: Option<&'a ChatMessageCursor>,
        limit: u32,
    ) -> BoxFuture<'a, Result<ChatMessagePage, StoreError>>;

    /// Edit an author's message within the policy window, recording its prior
    /// body in the revision audit table in the same transaction.
    fn edit_message<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
        content: &'a str,
        mentions: &'a [ActorId],
        now: OffsetDateTime,
    ) -> BoxFuture<'a, Result<ChatMessage, StoreError>>;

    /// Soft-delete an author's message within the policy window, or any
    /// message when the actor is a room manager.
    fn delete_message<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
        message_id: ChatMessageId,
        now: OffsetDateTime,
    ) -> BoxFuture<'a, Result<ChatMessage, StoreError>>;
}
