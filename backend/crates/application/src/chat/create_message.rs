use crate::{chat::ChatOperationError, ports::ChatStore};
use mahfel_domain::{
    ActorId, ChatChannel, ChatChannelId, ChatMessage, ChatMessageDraft, ChatMessageId, RoomId,
};
use time::OffsetDateTime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateChatMessage {
    pub room_id: RoomId,
    pub author_id: ActorId,
    pub content: String,
    pub parent_id: Option<ChatMessageId>,
    pub mentions: Vec<ActorId>,
}

/// Persist a message in the room's durable chat channel.
pub async fn create_chat_message<C>(
    chat: &C,
    command: CreateChatMessage,
) -> Result<ChatMessage, ChatOperationError>
where
    C: ChatStore + ?Sized,
{
    let now = OffsetDateTime::now_utc();
    let channel = ChatChannel::for_room(ChatChannelId::new(), command.room_id, now);
    let draft = ChatMessageDraft::new(
        ChatMessageId::new(),
        command.author_id,
        command.content,
        now,
    )?
    .with_details(command.parent_id, command.mentions)?;
    Ok(chat.create_message(&channel, &draft).await?)
}
