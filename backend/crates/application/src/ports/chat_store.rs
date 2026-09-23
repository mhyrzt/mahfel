use super::{BoxFuture, StoreError};
use mahfel_domain::{ChatChannel, ChatMessage, ChatMessageDraft};

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
}
