use crate::{chat::ChatOperationError, ports::ChatStore};
use mahfel_domain::{ActorId, ChatMessage, ChatMessageId, RoomId};
use time::OffsetDateTime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeleteChatMessage {
    pub room_id: RoomId,
    pub actor_id: ActorId,
    pub message_id: ChatMessageId,
}

pub async fn delete_chat_message<C>(
    chat: &C,
    command: DeleteChatMessage,
) -> Result<ChatMessage, ChatOperationError>
where
    C: ChatStore + ?Sized,
{
    Ok(chat
        .delete_message(
            command.room_id,
            command.actor_id,
            command.message_id,
            OffsetDateTime::now_utc(),
        )
        .await?)
}
