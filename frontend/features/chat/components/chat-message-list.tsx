import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { roomCopy } from "@/features/rooms/room-copy";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { MessageReactions } from "./message-reactions";

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
    <div className="space-y-4">
      {cursor && onLoadOlder ? (
        <Button onClick={() => onLoadOlder(cursor)} size="sm" variant="outline">
          {text.olderMessages}
        </Button>
      ) : null}
      <ol className="space-y-3" data-testid="chat-messages">
        {messages.map((message) => (
          <li className="border-b pb-3 last:border-0" key={message.id}>
            <p className="text-xs text-muted-foreground">
              {message.author_id === actorId
                ? text.you
                : `${text.participant} ${message.author_id.slice(0, 4)}`}
            </p>
            {editingId === message.id ? (
              <div className="mt-2 space-y-2">
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
              <p className="mt-1 whitespace-pre-wrap text-sm">
                {message.content ?? text.removedMessage}
              </p>
            )}
            {message.deleted_at === null || message.deleted_at === undefined ? (
              <div className="mt-2 flex flex-wrap gap-1">
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
              </div>
            ) : null}
            {replies[message.id] ? (
              <ol
                className="ms-4 mt-3 space-y-2 border-s ps-4"
                data-testid={`chat-replies-${message.id}`}
              >
                {replies[message.id].map((reply) => (
                  <li key={reply.id} className="text-sm">
                    <span className="me-2 text-xs text-muted-foreground">
                      {reply.author_id === actorId
                        ? text.you
                        : `${text.participant} ${reply.author_id.slice(0, 4)}`}
                    </span>
                    <span className="whitespace-pre-wrap">
                      {reply.content ?? text.removedMessage}
                    </span>
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
