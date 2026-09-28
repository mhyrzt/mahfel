"use client";

import { useEffect, useRef } from "react";

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
  return (
    <div
      aria-label={text.chatHistory}
      className="min-h-0 flex-1 space-y-5 overflow-y-auto overscroll-contain pe-1"
      ref={logRef}
      role="log"
    >
      {cursor && onLoadOlder ? (
        <Button onClick={() => onLoadOlder(cursor)} size="sm" variant="outline">
          {text.olderMessages}
        </Button>
      ) : null}
      <ol data-testid="chat-messages">
        {messages.map((message, index) => {
          const own = message.author_id === actorId;
          const grouped = index > 0 && messages[index - 1].author_id === message.author_id;
          const groupCorner = own ? "rounded-se-md" : "rounded-ss-md";
          const sentAt = new Intl.DateTimeFormat(locale === "fa" ? "fa-IR" : "en-US", {
            hour: "2-digit",
            minute: "2-digit",
          }).format(new Date(message.created_at));
          return (
            <li key={message.id} className={grouped ? "mt-1" : "mt-4"}>
              <Message align={own ? "end" : "start"} className="group/message">
                <MessageAvatar className={grouped ? "invisible" : undefined}>
                  <Avatar className="size-8 text-xs">
                    <AvatarFallback>
                      {message.author_id === actorId
                        ? text.you.slice(0, 2)
                        : message.author_id.slice(0, 2).toUpperCase()}
                    </AvatarFallback>
                  </Avatar>
                </MessageAvatar>
                <MessageContent className={own ? "items-end" : undefined}>
                  {!grouped ? (
                    <MessageHeader>
                      {message.author_id === actorId
                        ? text.you
                        : `${text.participant} ${message.author_id.slice(0, 4)}`}
                    </MessageHeader>
                  ) : null}
                  <Bubble
                    variant={own ? "default" : "secondary"}
                    align={own ? "end" : "start"}
                    className={`max-w-full ${grouped ? groupCorner : ""}`}
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
                    <time
                      className={`mt-1 block text-end text-[10px] tabular-nums ${own ? "text-primary-foreground/70" : "text-muted-foreground"}`}
                      dateTime={message.created_at}
                    >
                      {sentAt}
                    </time>
                  </Bubble>
                  {message.deleted_at === null || message.deleted_at === undefined ? (
                    <MessageFooter
                      className={`${own ? "justify-end" : ""} transition-opacity [@media(hover:hover)]:opacity-0 [@media(hover:hover)]:group-hover/message:opacity-100 [@media(hover:hover)]:group-focus-within/message:opacity-100`}
                    >
                      {onReply ? (
                        <Button onClick={() => onReply(message.id)} size="sm" variant="ghost">
                          {text.reply}
                        </Button>
                      ) : null}
                      {onToggleReplies ? (
                        <Button
                          onClick={() => onToggleReplies(message.id)}
                          size="sm"
                          variant="ghost"
                        >
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
          );
        })}
      </ol>
    </div>
  );
}
