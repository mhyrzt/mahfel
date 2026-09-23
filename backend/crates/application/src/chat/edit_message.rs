use crate::{chat::ChatOperationError, ports::ChatStore};
use mahfel_domain::{ActorId, ChatMessage, ChatMessageDraft, ChatMessageId, RoomId};
use time::OffsetDateTime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditChatMessage {
    pub room_id: RoomId,
    pub actor_id: ActorId,
    pub message_id: ChatMessageId,
    pub content: String,
}

pub async fn edit_chat_message<C>(
    chat: &C,
    command: EditChatMessage,
) -> Result<ChatMessage, ChatOperationError>
where
    C: ChatStore + ?Sized,
{
    let now = OffsetDateTime::now_utc();
    ChatMessageDraft::new(
        command.message_id,
        command.actor_id,
        command.content.clone(),
        now,
    )?;
    Ok(chat
        .edit_message(
            command.room_id,
            command.actor_id,
            command.message_id,
            &command.content,
            now,
        )
        .await?)
}
