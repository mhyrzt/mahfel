import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Bubble, BubbleContent } from "@/components/ui/bubble";
import {
  Message,
  MessageAvatar,
  MessageContent,
  MessageFooter,
  MessageHeader,
} from "@/components/ui/message";
import { Textarea } from "@/components/ui/textarea";
import { roomCopy } from "@/features/rooms/room-copy";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { MessageReactions } from "./message-reactions";
import { MessageFiles } from "./message-files";

type Message = components["schemas"]["ChatMessageResponse"];

export function ChatMessageList({
  messages,
  roomId,
  locale,
  actorId,
  canModerate = false,
  loading = false,
  cursor = null,
  editingId = null,
  editDraft = "",
  replies = {},
  onLoadOlder,
  onReply,
  onToggleReplies,
  onEdit,
  onEditDraftChange,
  onSaveEdit,
  onCancelEdit,
  onRemove,
  showReactions = false,
}: Readonly<{
  messages: Message[];
  roomId: string;
  locale: Locale;
  actorId?: string;
  canModerate?: boolean;
  loading?: boolean;
  cursor?: string | null;
  editingId?: string | null;
  editDraft?: string;
  replies?: Record<string, Message[]>;
  onLoadOlder?: (cursor: string) => void;
  onReply?: (messageId: string) => void;
  onToggleReplies?: (messageId: string) => void;
  onEdit?: (message: Message) => void;
  onEditDraftChange?: (content: string) => void;
  onSaveEdit?: (messageId: string) => void;
  onCancelEdit?: () => void;
  onRemove?: (messageId: string) => void;
  showReactions?: boolean;
}>) {
  const text = roomCopy(locale);
  if (loading) {
    return <p className="text-sm text-muted-foreground">{text.loadingMessages}</p>;
  }
  if (messages.length === 0) {
    return (
      <p className="rounded-lg border border-dashed px-4 py-10 text-center text-sm text-muted-foreground">
        {text.noMessages}
      </p>
    );
  }
  return (
    <div
      aria-label={text.chatHistory}
      className="max-h-[36rem] space-y-5 overflow-y-auto overscroll-contain pe-2"
      role="log"
    >
      {cursor && onLoadOlder ? (
        <Button onClick={() => onLoadOlder(cursor)} size="sm" variant="outline">
          {text.olderMessages}
        </Button>
      ) : null}
      <ol className="space-y-5" data-testid="chat-messages">
        {messages.map((message) => (
          <li key={message.id}>
            <Message align={message.author_id === actorId ? "end" : "start"}>
              <MessageAvatar>
                <Avatar className="size-8 text-xs">
                  <AvatarFallback>
                    {message.author_id === actorId
                      ? text.you.slice(0, 2)
                      : message.author_id.slice(0, 2).toUpperCase()}
                  </AvatarFallback>
                </Avatar>
              </MessageAvatar>
              <MessageContent className={message.author_id === actorId ? "items-end" : undefined}>
                <MessageHeader>
                  {message.author_id === actorId
                    ? text.you
                    : `${text.participant} ${message.author_id.slice(0, 4)}`}
                </MessageHeader>
                <Bubble
                  className={
                    message.author_id === actorId ? "border-primary/20 bg-primary/10" : undefined
                  }
                >
                  <BubbleContent>
                    {editingId === message.id ? (
                      <div className="space-y-2">
                        <Textarea
                          aria-label={text.editMessage}
                          onChange={(event) => onEditDraftChange?.(event.target.value)}
                          value={editDraft}
                        />
                        <div className="flex gap-2">
                          <Button onClick={() => onSaveEdit?.(message.id)} size="sm">
                            {text.save}
                          </Button>
                          <Button onClick={onCancelEdit} size="sm" variant="ghost">
                            {text.cancel}
                          </Button>
                        </div>
                      </div>
                    ) : (
                      (message.content ?? text.removedMessage)
                    )}
                  </BubbleContent>
                </Bubble>
                {message.deleted_at === null || message.deleted_at === undefined ? (
                  <MessageFooter
                    className={message.author_id === actorId ? "justify-end" : undefined}
                  >
                    {onReply ? (
                      <Button onClick={() => onReply(message.id)} size="sm" variant="ghost">
                        {text.reply}
                      </Button>
                    ) : null}
                    {onToggleReplies ? (
                      <Button onClick={() => onToggleReplies(message.id)} size="sm" variant="ghost">
                        {text.replies}
                      </Button>
                    ) : null}
                    {showReactions ? (
                      <MessageReactions roomId={roomId} messageId={message.id} locale={locale} />
                    ) : null}
                    {message.author_id === actorId && onEdit ? (
                      <Button onClick={() => onEdit(message)} size="sm" variant="ghost">
                        {text.editMessage}
                      </Button>
                    ) : null}
                    {(message.author_id === actorId || canModerate) && onRemove ? (
                      <Button onClick={() => onRemove(message.id)} size="sm" variant="ghost">
                        {text.deleteMessage}
                      </Button>
                    ) : null}
                    <MessageFiles
                      canAttach={message.author_id === actorId}
                      locale={locale}
                      messageId={message.id}
                    />
                  </MessageFooter>
                ) : null}
              </MessageContent>
            </Message>
            {replies[message.id] ? (
              <ol
                className="ms-10 mt-3 space-y-3 border-s ps-4"
                data-testid={`chat-replies-${message.id}`}
              >
                {replies[message.id].map((reply) => (
                  <li key={reply.id}>
                    <Message align={reply.author_id === actorId ? "end" : "start"}>
                      <MessageAvatar>
                        <Avatar className="size-7 text-xs">
                          <AvatarFallback>
                            {reply.author_id.slice(0, 2).toUpperCase()}
                          </AvatarFallback>
                        </Avatar>
                      </MessageAvatar>
                      <MessageContent>
                        <MessageHeader>
                          {reply.author_id === actorId
                            ? text.you
                            : `${text.participant} ${reply.author_id.slice(0, 4)}`}
                        </MessageHeader>
                        <Bubble>
                          <BubbleContent>{reply.content ?? text.removedMessage}</BubbleContent>
                        </Bubble>
                        {reply.deleted_at ? null : (
                          <MessageFooter>
                            <MessageFiles
                              canAttach={reply.author_id === actorId}
                              locale={locale}
                              messageId={reply.id}
                            />
                          </MessageFooter>
                        )}
                      </MessageContent>
                    </Message>
                  </li>
                ))}
              </ol>
            ) : null}
          </li>
        ))}
      </ol>
    </div>
  );
}
