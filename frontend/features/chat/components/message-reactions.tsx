"use client";

import { useState } from "react";
import { SmilePlus } from "lucide-react";

import { Button } from "@/components/ui/button";
import { roomCopy } from "@/features/rooms/room-copy";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

type Reactions = components["schemas"]["ReactionsResponse"];
const choices = ["👍", "👎", "❤️", "😂", "😮", "😢"];

export function MessageReactions({
  roomId,
  messageId,
  locale,
}: Readonly<{ roomId: string; messageId: string; locale: Locale }>) {
  const text = roomCopy(locale);
  const [open, setOpen] = useState(false);
  const [reactions, setReactions] = useState<Reactions | null>(null);
  const [error, setError] = useState(false);
  const path = `/v1/rooms/${encodeURIComponent(roomId)}/messages/${encodeURIComponent(messageId)}/reaction`;

  async function refresh() {
    const { data } = await browserApiClient.get<Reactions>(path);
    setReactions(data);
  }

  async function toggle() {
    if (open) {
      setOpen(false);
      return;
    }
    setOpen(true);
    try {
      await refresh();
      setError(false);
    } catch {
      setError(true);
    }
  }

  async function choose(emoji: string) {
    try {
      if (reactions?.my_reaction === emoji) {
        await browserApiClient.delete(path);
      } else {
        await browserApiClient.put(path, { emoji });
      }
      await refresh();
      setError(false);
    } catch {
      setError(true);
    }
  }

  return (
    <div data-testid={`reactions-${messageId}`}>
      <Button
        aria-expanded={open}
        aria-label={text.react}
        className="size-6 text-current opacity-70 hover:opacity-100"
        onClick={() => void toggle()}
        size="icon"
        type="button"
        variant="ghost"
      >
        <SmilePlus aria-hidden="true" className="size-4" />
      </Button>
      {open ? (
        <div className="mt-1 flex flex-wrap items-center gap-1">
          {choices.map((emoji) => {
            const count = reactions?.counts.find((item) => item.emoji === emoji)?.count ?? 0;
            return (
              <Button
                aria-label={`${text.react} ${emoji}`}
                aria-pressed={reactions?.my_reaction === emoji}
                key={emoji}
                onClick={() => void choose(emoji)}
                size="sm"
                variant={reactions?.my_reaction === emoji ? "secondary" : "ghost"}
              >
                {emoji}
                {count > 0 ? ` ${count}` : ""}
              </Button>
            );
          })}
          {error ? (
            <span className="text-xs text-destructive" role="alert">
              {text.reactionFailed}
            </span>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
