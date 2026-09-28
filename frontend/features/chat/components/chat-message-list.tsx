"use client";

import { useEffect, useRef } from "react";
import { MoreVertical } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Bubble, BubbleContent } from "@/components/ui/bubble";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Message, MessageAvatar, MessageContent } from "@/components/ui/message";
import { Textarea } from "@/components/ui/textarea";
import { roomCopy } from "@/features/rooms/room-copy";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { MessageAttachments } from "./message-attachments";
import { MessageReactions } from "./message-reactions";

type Message = components["schemas"]["ChatMessageResponse"];
type Copy = ReturnType<typeof roomCopy>;

function authorLabel(message: Message, actorId: string | undefined, text: Copy): string {
  return message.author_id === actorId
    ? text.you
    : `${text.participant} ${message.author_id.slice(0, 4)}`;
}

function authorInitials(authorId: string): string {
  return authorId.slice(0, 2).toUpperCase();
}

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
  const logRef = useRef<HTMLDivElement>(null);
  const lastIdRef = useRef<string | null>(null);
  const lastId = messages.at(-1)?.id ?? null;
  useEffect(() => {
    const log = logRef.current;
    if (!log || !lastId || lastIdRef.current === lastId) {
      return;
    }
    const nearBottom = log.scrollHeight - log.scrollTop - log.clientHeight < 160;
    if (lastIdRef.current === null || nearBottom) {
      log.scrollTop = log.scrollHeight;
    }
    lastIdRef.current = lastId;
  }, [lastId]);
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
  const sentAt = (value: string) =>
    new Intl.DateTimeFormat(locale === "fa" ? "fa-IR" : "en-US", {
      hour: "2-digit",
      minute: "2-digit",
    }).format(new Date(value));

  const actions = (message: Message, own: boolean) => (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          aria-label={text.messageActions}
          className="size-6 text-current opacity-70 hover:opacity-100"
          size="icon"
          type="button"
          variant="ghost"
        >
          <MoreVertical aria-hidden="true" className="size-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align={own ? "end" : "start"}>
        {onReply ? (
          <DropdownMenuItem onSelect={() => onReply(message.id)}>{text.reply}</DropdownMenuItem>
        ) : null}
        {onToggleReplies ? (
          <DropdownMenuItem onSelect={() => onToggleReplies(message.id)}>
            {text.replies}
          </DropdownMenuItem>
        ) : null}
        {own && onEdit ? (
          <DropdownMenuItem onSelect={() => onEdit(message)}>{text.editMessage}</DropdownMenuItem>
        ) : null}
        {(own || canModerate) && onRemove ? (
          <DropdownMenuItem
            className="text-destructive focus:text-destructive"
            onSelect={() => onRemove(message.id)}
          >
            {text.deleteMessage}
          </DropdownMenuItem>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  );

  return (
    <div
      aria-label={text.chatHistory}
      className="min-h-0 flex-1 space-y-3 overflow-y-auto overscroll-contain pe-1"
      ref={logRef}
      role="log"
    >
      {cursor && onLoadOlder ? (
        <Button onClick={() => onLoadOlder(cursor)} size="sm" variant="outline">
          {text.olderMessages}
        </Button>
      ) : null}
      <ol className="space-y-3" data-testid="chat-messages">
        {messages.map((message, index) => {
          const own = message.author_id === actorId;
          const grouped = index > 0 && messages[index - 1].author_id === message.author_id;
          const deleted = message.deleted_at !== null && message.deleted_at !== undefined;
          return (
            <li key={message.id}>
              <Message align={own ? "end" : "start"} className="group/message gap-2">
                <MessageAvatar className={grouped ? "invisible" : undefined}>
                  <Avatar className="size-8 text-xs">
                    <AvatarFallback>{authorInitials(message.author_id)}</AvatarFallback>
                  </Avatar>
                </MessageAvatar>
                <MessageContent className={own ? "w-full items-end" : "w-full"}>
                  <Bubble
                    align={own ? "end" : "start"}
                    className="w-full"
                    variant={own ? "default" : "secondary"}
                  >
                    <div className="mb-1 flex items-center gap-1">
                      <span
                        className={`truncate text-xs font-semibold opacity-70 ${grouped ? "invisible" : ""}`}
                      >
                        {authorLabel(message, actorId, text)}
                      </span>
                      <span className="ms-auto flex items-center gap-0.5">
                        {showReactions && !deleted ? (
                          <MessageReactions
                            locale={locale}
                            messageId={message.id}
                            roomId={roomId}
                          />
                        ) : null}
                        {!deleted ? actions(message, own) : null}
                      </span>
                    </div>
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
                      <BubbleContent>{message.content ?? text.removedMessage}</BubbleContent>
                    )}
                    {!deleted ? (
                      <MessageAttachments files={message.attachments ?? []} locale={locale} />
                    ) : null}
                    <time
                      className={`mt-1 block text-end text-[10px] tabular-nums ${own ? "text-primary-foreground/70" : "text-muted-foreground"}`}
                      dateTime={message.created_at}
                    >
                      {sentAt(message.created_at)}
                    </time>
                  </Bubble>
                </MessageContent>
              </Message>
              {replies[message.id] ? (
                <ol
                  className="ms-10 mt-2 space-y-2 border-s ps-4"
                  data-testid={`chat-replies-${message.id}`}
                >
                  {replies[message.id].map((reply) => {
                    const replyOwn = reply.author_id === actorId;
                    return (
                      <li key={reply.id}>
                        <Message align={replyOwn ? "end" : "start"} className="gap-2">
                          <MessageAvatar>
                            <Avatar className="size-7 text-xs">
                              <AvatarFallback>{authorInitials(reply.author_id)}</AvatarFallback>
                            </Avatar>
                          </MessageAvatar>
                          <MessageContent className={replyOwn ? "w-full items-end" : "w-full"}>
                            <Bubble
                              className="w-full"
                              variant={replyOwn ? "default" : "secondary"}
                              align={replyOwn ? "end" : "start"}
                            >
                              <p className="mb-1 truncate text-xs font-semibold opacity-70">
                                {authorLabel(reply, actorId, text)}
                              </p>
                              <BubbleContent>{reply.content ?? text.removedMessage}</BubbleContent>
                              {reply.deleted_at ? null : (
                                <MessageAttachments
                                  files={reply.attachments ?? []}
                                  locale={locale}
                                />
                              )}
                            </Bubble>
                          </MessageContent>
                        </Message>
                      </li>
                    );
                  })}
                </ol>
              ) : null}
            </li>
          );
        })}
      </ol>
    </div>
  );
}
