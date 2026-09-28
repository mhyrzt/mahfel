use crate::{chat::ChatOperationError, ports::ChatStore};
use mahfel_domain::{ActorId, ChatMessageCursor, ChatMessagePage, RoomId, SessionId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListChatMessages {
    pub room_id: RoomId,
    pub actor_id: ActorId,
    pub before: Option<ChatMessageCursor>,
    pub limit: u32,
    /// `None` lists the general room conversation; `Some` lists one session.
    pub session_id: Option<SessionId>,
}

pub async fn list_chat_messages<C>(
    chat: &C,
    query: ListChatMessages,
) -> Result<ChatMessagePage, ChatOperationError>
where
    C: ChatStore + ?Sized,
{
    if !(1..=100).contains(&query.limit) {
        return Err(ChatOperationError::InvalidPageSize);
    }
    Ok(chat
        .list_messages(
            query.room_id,
            query.actor_id,
            query.session_id,
            query.before.as_ref(),
            query.limit,
        )
        .await?)
}
