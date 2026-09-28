"use client";

import { useEffect, useState } from "react";
import { UsersRound } from "lucide-react";

import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";

const REFRESH_MS = 20_000;

export function SessionParticipantCount({
  sessionId,
  live,
  locale,
}: Readonly<{ sessionId: string; live: boolean; locale: Locale }>) {
  const text = roomCopy(locale);
  const [count, setCount] = useState<number | null>(null);

  useEffect(() => {
    if (!live) {
      return undefined;
    }
    let active = true;
    const refresh = async () => {
      try {
        const { data } = await browserApiClient.get<unknown>(
          `/v1/sessions/${encodeURIComponent(sessionId)}/roster`,
        );
        if (active) {
          setCount(Array.isArray(data) ? data.length : null);
        }
      } catch {
        if (active) {
          setCount(null);
        }
      }
    };
    void refresh();
    const timer = window.setInterval(() => void refresh(), REFRESH_MS);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, [live, sessionId]);

  if (!live || count === null) {
    return null;
  }
  return (
    <span
      className="inline-flex items-center gap-1 text-xs text-muted-foreground"
      data-testid={`session-participants-${sessionId}`}
    >
      <UsersRound aria-hidden="true" className="size-3.5" />
      {count} {text.inSession}
    </span>
  );
}
