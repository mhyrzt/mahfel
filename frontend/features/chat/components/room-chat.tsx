"use client";

import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";

import { uploadFile } from "@/features/files/client";
import { roomCopy } from "@/features/rooms/room-copy";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";
import { SignalingClient } from "@/lib/websocket/client";
import { ChatMessageList } from "./chat-message-list";
import { ConversationPanel } from "./conversation-panel";

type Message = components["schemas"]["ChatMessageResponse"];
type History = components["schemas"]["ChatMessageHistoryResponse"];

function mergeMessages(existing: Message[], incoming: Message[], older = false): Message[] {
  const byId = new Map(existing.map((message) => [message.id, message]));
  for (const message of incoming) {
    byId.set(message.id, message);
  }
  const sorted = Array.from(byId.values()).sort(
    (a, b) => a.created_at.localeCompare(b.created_at) || a.id.localeCompare(b.id),
  );
  return older ? sorted.slice(0, 200) : sorted.slice(-200);
}

export function RoomChat({
  roomId,
  sessionId,
  locale,
  actorId,
  canModerate = false,
  compact = false,
}: Readonly<{
  roomId: string;
  sessionId?: string;
  locale: Locale;
  actorId?: string;
  canModerate?: boolean;
  compact?: boolean;
}>) {
  const text = roomCopy(locale);
  const [messages, setMessages] = useState<Message[]>([]);
  const [cursor, setCursor] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [attachments, setAttachments] = useState<File[]>([]);
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editDraft, setEditDraft] = useState("");
  const [replyTo, setReplyTo] = useState<string | null>(null);
  const [replies, setReplies] = useState<Record<string, Message[]>>({});
  const generationRef = useRef(0);
  const [reactionVersions, setReactionVersions] = useState<Record<string, number>>({});
  const refreshRef = useRef<Promise<void> | null>(null);
  const load = useCallback(
    async (kind: "initial" | "refresh" | "older", before?: string) => {
      const generation = generationRef.current;
      try {
        const { data } = await browserApiClient.get<History>(
          `/v1/rooms/${encodeURIComponent(roomId)}/messages`,
          {
            params: {
              limit: 50,
              ...(before ? { cursor: before } : {}),
              ...(sessionId ? { session_id: sessionId } : {}),
            },
          },
        );
        if (generation !== generationRef.current) {
          return;
        }
        setMessages((previous) =>
          kind === "initial"
            ? mergeMessages(data.messages, previous)
            : mergeMessages(previous, data.messages, kind === "older"),
        );
        if (kind !== "refresh") {
          setCursor(data.next_cursor ?? null);
        }
        setError(null);
      } catch {
        if (generation === generationRef.current) {
          setError(text.messageUnavailable);
        }
      } finally {
        if (generation === generationRef.current) {
          setLoading(false);
        }
      }
    },
    [roomId, sessionId, text.messageUnavailable],
  );

  useEffect(() => {
    const generation = ++generationRef.current;
    let sequence = 0;
    const attachmentLoads = new Map<string, number>();
    const refresh = () => {
      if (!refreshRef.current) {
        const pending = load("refresh");
        refreshRef.current = pending;
        void pending.finally(() => {
          if (refreshRef.current === pending) {
            refreshRef.current = null;
          }
        });
      }
    };
    void Promise.resolve().then(async () => {
      if (generation !== generationRef.current) {
        return;
      }
      setMessages([]);
      setReplies({});
      setReactionVersions({});
      setCursor(null);
      setLoading(true);
      await load("initial");
    });
    const client = new SignalingClient(
      null,
      (message) => {
        if (generation !== generationRef.current) {
          return;
        }
        if (
          message.type === "chat_subscribed" &&
          message.room_id === roomId &&
          document.visibilityState === "visible"
        ) {
          void browserApiClient
            .put(`/v1/rooms/${encodeURIComponent(roomId)}/read-marker`, {
              sequence: message.head_sequence,
            })
            .catch(() => undefined);
        }
        if (message.type === "chat_event" && message.event.room_id === roomId) {
          const event = message.event;
          if (event.sequence <= sequence) {
            return;
          }
          if (sequence > 0 && event.sequence !== sequence + 1) {
            client.send({ type: "unsubscribe_chat", room_id: roomId });
            client.send({ type: "subscribe_chat", room_id: roomId, after_sequence: sequence });
            return;
          }
          sequence = event.sequence;
          const scoped = event.session_id ?? null;
          if (scoped !== (sessionId ?? null)) {
            return;
          }
          if (event.kind === "reaction_set" || event.kind === "reaction_removed") {
            setReactionVersions((previous) =>
              Object.fromEntries(
                Object.entries({ ...previous, [event.message_id]: event.sequence }).slice(-200),
              ),
            );
          } else if (
            event.message_id &&
            event.author_id &&
            Number.isFinite(Date.parse(event.created_at))
          ) {
            const apply = (previous: Message[]) => {
              const existing = previous.find((item) => item.id === event.message_id);
              const updated: Message = {
                ...existing,
                id: event.message_id,
                channel_id: existing?.channel_id ?? "",
                author_id: event.author_id,
                content: event.content,
                created_at: event.created_at,
                edited_at: event.edited_at,
                deleted_at: event.deleted_at,
                parent_id: event.parent_id,
                session_id: event.session_id,
                attachments:
                  event.deleted_at || event.attachment_ids?.length === 0
                    ? []
                    : existing?.attachments,
              };
              return mergeMessages(previous, [updated]);
            };
            if (event.parent_id) {
              const parent = event.parent_id;
              setReplies((previous) =>
                previous[parent] ? { ...previous, [parent]: apply(previous[parent]) } : previous,
              );
            } else {
              setMessages(apply);
            }
            attachmentLoads.set(event.message_id, event.sequence);
            if (attachmentLoads.size > 200) {
              const oldest = attachmentLoads.keys().next().value;
              if (oldest) {
                attachmentLoads.delete(oldest);
              }
            }
            if (event.attachment_ids?.length && !event.deleted_at) {
              void browserApiClient
                .get<components["schemas"]["FileListResponse"]>("/v1/files", {
                  params: { context_kind: "chat_message", context_id: event.message_id, limit: 4 },
                })
                .then(({ data }) => {
                  if (
                    generation !== generationRef.current ||
                    attachmentLoads.get(event.message_id) !== event.sequence
                  ) {
                    return;
                  }
                  const attach = (previous: Message[]) =>
                    previous.map((item) =>
                      item.id === event.message_id && !item.deleted_at
                        ? { ...item, attachments: data.files }
                        : item,
                    );
                  if (event.parent_id) {
                    const parent = event.parent_id;
                    setReplies((previous) =>
                      previous[parent]
                        ? { ...previous, [parent]: attach(previous[parent]) }
                        : previous,
                    );
                  } else {
                    setMessages(attach);
                  }
                })
                .catch(() => undefined);
            }
          } else {
            refresh();
          }
          if (document.visibilityState === "visible") {
            void browserApiClient
              .put(`/v1/rooms/${encodeURIComponent(roomId)}/read-marker`, {
                sequence: event.sequence,
              })
              .catch(() => undefined);
          }
        }
      },
      (status) => {
        if (status === "connected") {
          client.send({ type: "subscribe_chat", room_id: roomId, after_sequence: sequence });
        }
      },
    );
    client.connect();
    return () => {
      generationRef.current += 1;
      refreshRef.current = null;
      client.close();
    };
  }, [roomId, sessionId, load]);

  async function send(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const content = draft.trim();
    if (!content || sending) {
      return;
    }
    setSending(true);
    const files = attachments;
    try {
      const target = `/v1/rooms/${encodeURIComponent(roomId)}/messages`;
      const { data } = await browserApiClient.post<Message>(
        replyTo ? `${target}/${encodeURIComponent(replyTo)}/replies` : target,
        {
          content,
          ...(sessionId && !replyTo ? { session_id: sessionId } : {}),
        },
      );
      if (files.length > 0) {
        data.attachments = [];
        for (const file of files) {
          data.attachments.push(await uploadFile(file, { kind: "chat_message", id: data.id }));
        }
      }
      if (replyTo) {
        const parentId = replyTo;
        setReplies((previous) => ({
          ...previous,
          [parentId]: mergeMessages(previous[parentId] ?? [], [data]),
        }));
      } else {
        setMessages((previous) => mergeMessages(previous, [data]));
      }
      setDraft("");
      setAttachments([]);
      setReplyTo(null);
      setError(null);
    } catch {
      setError(text.messageFailed);
    } finally {
      setSending(false);
    }
  }

  async function saveEdit(messageId: string) {
    if (!editDraft.trim()) {
      return;
    }
    try {
      const { data } = await browserApiClient.patch<Message>(
        `/v1/rooms/${encodeURIComponent(roomId)}/messages/${encodeURIComponent(messageId)}`,
        { content: editDraft.trim() },
      );
      setMessages((previous) => mergeMessages(previous, [data]));
      setEditingId(null);
      setError(null);
    } catch {
      setError(text.messageActionFailed);
    }
  }

  async function remove(messageId: string) {
    try {
      const { data } = await browserApiClient.delete<Message>(
        `/v1/rooms/${encodeURIComponent(roomId)}/messages/${encodeURIComponent(messageId)}`,
      );
      setMessages((previous) => mergeMessages(previous, [data]));
      setError(null);
    } catch {
      setError(text.messageActionFailed);
    }
  }

  async function showReplies(messageId: string) {
    if (replies[messageId]) {
      setReplies((previous) => {
        const next = { ...previous };
        delete next[messageId];
        return next;
      });
      return;
    }
    try {
      const { data } = await browserApiClient.get<History>(
        `/v1/rooms/${encodeURIComponent(roomId)}/messages/${encodeURIComponent(messageId)}/replies`,
        { params: { limit: 50 } },
      );
      setReplies((previous) => ({ ...previous, [messageId]: data.messages }));
    } catch {
      setError(text.messageUnavailable);
    }
  }

  return (
    <ConversationPanel
      roomId={roomId}
      locale={locale}
      compact={compact}
      draft={draft}
      onDraftChange={setDraft}
      onSend={send}
      sending={sending}
      replying={Boolean(replyTo)}
      onCancelReply={() => setReplyTo(null)}
      attachments={attachments}
      onAttachFiles={(files) => setAttachments((previous) => [...previous, ...files])}
      onRemoveAttachment={(index) =>
        setAttachments((previous) => previous.filter((_, position) => position !== index))
      }
      error={error}
    >
      <ChatMessageList
        messages={messages}
        roomId={roomId}
        locale={locale}
        actorId={actorId}
        canModerate={canModerate}
        loading={loading}
        cursor={cursor}
        editingId={editingId}
        editDraft={editDraft}
        replies={replies}
        reactionVersions={reactionVersions}
        onLoadOlder={(before) => void load("older", before)}
        onReply={(messageId) => {
          setReplyTo(messageId);
          setReplies((previous) => ({ ...previous, [messageId]: previous[messageId] ?? [] }));
        }}
        onToggleReplies={(messageId) => void showReplies(messageId)}
        onEdit={(message) => {
          setEditingId(message.id);
          setEditDraft(message.content ?? "");
        }}
        onEditDraftChange={setEditDraft}
        onSaveEdit={(messageId) => void saveEdit(messageId)}
        onCancelEdit={() => setEditingId(null)}
        onRemove={(messageId) => void remove(messageId)}
        showReactions
      />
    </ConversationPanel>
  );
}
