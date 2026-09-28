"use client";

import { useCallback, useEffect, useState, type FormEvent } from "react";

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
  locale,
  actorId,
  canModerate = false,
  compact = false,
}: Readonly<{
  roomId: string;
  locale: Locale;
  actorId?: string;
  canModerate?: boolean;
  compact?: boolean;
}>) {
  const text = roomCopy(locale);
  const [messages, setMessages] = useState<Message[]>([]);
  const [cursor, setCursor] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editDraft, setEditDraft] = useState("");
  const [replyTo, setReplyTo] = useState<string | null>(null);
  const [replies, setReplies] = useState<Record<string, Message[]>>({});
  const load = useCallback(
    async (kind: "initial" | "refresh" | "older", before?: string) => {
      try {
        const { data } = await browserApiClient.get<History>(
          `/v1/rooms/${encodeURIComponent(roomId)}/messages`,
          { params: before ? { cursor: before, limit: 50 } : { limit: 50 } },
        );
        setMessages((previous) => mergeMessages(previous, data.messages, kind === "older"));
        if (kind !== "refresh") {
          setCursor(data.next_cursor ?? null);
        }
        setError(null);
      } catch {
        setError(text.messageUnavailable);
      } finally {
        setLoading(false);
      }
    },
    [roomId, text.messageUnavailable],
  );

  useEffect(() => {
    void Promise.resolve().then(() => load("initial"));
    const client = new SignalingClient(
      null,
      (message) => {
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
          void load("refresh");
          if (document.visibilityState === "visible") {
            void browserApiClient
              .put(`/v1/rooms/${encodeURIComponent(roomId)}/read-marker`, {
                sequence: message.event.sequence,
              })
              .catch(() => undefined);
          }
        }
      },
      (status) => {
        if (status === "connected") {
          client.send({ type: "subscribe_chat", room_id: roomId, after_sequence: 0 });
        }
      },
    );
    client.connect();
    return () => {
      client.close();
    };
  }, [roomId, load]);

  async function send(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const content = draft.trim();
    if (!content || sending) {
      return;
    }
    setSending(true);
    try {
      const target = `/v1/rooms/${encodeURIComponent(roomId)}/messages`;
      const { data } = await browserApiClient.post<Message>(
        replyTo ? `${target}/${encodeURIComponent(replyTo)}/replies` : target,
        { content },
      );
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
