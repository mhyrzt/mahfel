use super::{BoxFuture, StoreError};
use mahfel_domain::{
    ActorId, ChatChannel, ChatMessage, ChatMessageCursor, ChatMessageDraft, ChatMessageId,
    ChatMessagePage, RoomId,
};
use time::OffsetDateTime;

/// Persistence for room-scoped chat channels and their durable messages.
pub trait ChatStore: Send + Sync {
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
    /// is an exclusive `(created_at, id)` keyset boundary.
    fn list_messages<'a>(
        &'a self,
        room_id: RoomId,
        actor_id: ActorId,
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
