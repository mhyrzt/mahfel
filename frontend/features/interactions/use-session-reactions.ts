"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { isAxiosError } from "axios";
import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import type { ServerMessage } from "@/lib/websocket/generated/messages";
import type { LiveReaction } from "./reaction-overlay";

export function useSessionReactions(sessionId: string, locale: Locale, enabled: boolean) {
  const [reactions, setReactions] = useState<LiveReaction[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const busyRef = useRef(false);
  const active = useRef(enabled);
  const generationRef = useRef(0);
  const timers = useRef(new Map<string, ReturnType<typeof setTimeout>>());
  const seen = useRef(new Set<string>());
  const text = messagesFor(locale).interactions;
  useEffect(() => {
    const generation = ++generationRef.current;
    active.current = enabled;
    busyRef.current = false;
    const ownedTimers = timers.current;
    seen.current.clear();
    void Promise.resolve().then(() => {
      if (generationRef.current === generation) {
        setReactions([]);
        setBusy(false);
        setError(null);
      }
    });
    return () => {
      active.current = false;
      for (const timer of ownedTimers.values()) {
        clearTimeout(timer);
      }
      ownedTimers.clear();
    };
  }, [enabled, sessionId]);
  const handleMessage = useCallback(
    (message: ServerMessage) => {
      if (
        !active.current ||
        message.type !== "session_reaction" ||
        message.session_id !== sessionId ||
        seen.current.has(message.event_id)
      ) {
        return;
      }
      seen.current.add(message.event_id);
      if (seen.current.size > 128) {
        const oldest = seen.current.values().next().value;
        if (oldest) {
          seen.current.delete(oldest);
        }
      }
      setReactions((previous) => [...previous.slice(-11), message]);
      const timer = setTimeout(() => {
        timers.current.delete(message.event_id);
        setReactions((previous) =>
          previous.filter((reaction) => reaction.event_id !== message.event_id),
        );
      }, 3_500);
      timers.current.set(message.event_id, timer);
      if (timers.current.size > 12) {
        const oldest = timers.current.keys().next().value;
        if (oldest) {
          clearTimeout(timers.current.get(oldest));
          timers.current.delete(oldest);
        }
      }
    },
    [sessionId],
  );
  const send = useCallback(
    async (emoji: string) => {
      if (!active.current || busyRef.current) {
        return;
      }
      busyRef.current = true;
      const generation = generationRef.current;
      setBusy(true);
      setError(null);
      try {
        await browserApiClient.post(
          `/v1/sessions/${encodeURIComponent(sessionId)}/interactions/reactions`,
          { emoji },
        );
      } catch (failure) {
        if (active.current && generationRef.current === generation) {
          setError(
            isAxiosError(failure) && failure.response?.status === 429
              ? text.reactionThrottled
              : text.failed,
          );
        }
      } finally {
        if (generationRef.current === generation) {
          busyRef.current = false;
          if (active.current) {
            setBusy(false);
          }
        }
      }
    },
    [sessionId, text.failed, text.reactionThrottled],
  );
  return { reactions, busy, error, handleMessage, send };
}
