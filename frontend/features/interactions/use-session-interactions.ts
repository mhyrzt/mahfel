"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { isAxiosError } from "axios";
import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import type { ServerMessage } from "@/lib/websocket/generated/messages";
import type { InteractionAction, InteractionsSnapshot } from "./types";

export function useSessionInteractions(sessionId: string, locale: Locale, enabled: boolean) {
  const [snapshot, setSnapshot] = useState<InteractionsSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const versionRef = useRef(-1);
  const activeActivityRef = useRef<string | null | undefined>(undefined);
  const refreshRef = useRef<() => Promise<void>>(async () => {});
  const pendingRef = useRef<{ action: string; requestId: string } | null>(null);
  const busyRef = useRef(false);
  const contextRef = useRef({ sessionId, enabled });
  const text = messagesFor(locale).interactions;

  useEffect(() => {
    contextRef.current = { sessionId, enabled };
    if (!enabled) {
      return undefined;
    }
    let active = true;
    let inFlight = false;
    let requested = false;
    const controller = new AbortController();
    versionRef.current = -1;
    activeActivityRef.current = undefined;
    async function refresh() {
      if (inFlight) {
        requested = true;
        return;
      }
      inFlight = true;
      do {
        requested = false;
        try {
          const { data } = await browserApiClient.get<InteractionsSnapshot>(
            `/v1/sessions/${encodeURIComponent(sessionId)}/interactions`,
            { signal: controller.signal },
          );
          if (active && data.version >= versionRef.current) {
            versionRef.current = data.version;
            const activity = data.activities.find((item) => item.status === "open");
            if (
              activeActivityRef.current !== undefined &&
              activity &&
              activeActivityRef.current !== activity.id
            ) {
              setNotice(activity.id);
            }
            activeActivityRef.current = activity?.id ?? null;
            setSnapshot(data);
            setError(null);
          }
        } catch (failure) {
          if (active) {
            const status = isAxiosError(failure) ? failure.response?.status : undefined;
            if (status === 403 || status === 404) {
              setSnapshot(null);
            }
            setError(text.failed);
          }
        }
      } while (active && requested);
      inFlight = false;
    }
    refreshRef.current = refresh;
    void refresh();
    const timer = window.setInterval(() => void refresh(), 15_000);
    return () => {
      active = false;
      controller.abort();
      window.clearInterval(timer);
      refreshRef.current = async () => {};
    };
  }, [enabled, sessionId, text.failed]);

  const send = useCallback(
    async (action: InteractionAction) => {
      if (busyRef.current || !contextRef.current.enabled) {
        return false;
      }
      const targetSession = sessionId;
      busyRef.current = true;
      setBusy(true);
      setError(null);
      const fingerprint = JSON.stringify(action);
      const request =
        pendingRef.current?.action === fingerprint
          ? pendingRef.current
          : { action: fingerprint, requestId: crypto.randomUUID() };
      pendingRef.current = request;
      try {
        await browserApiClient.post(
          `/v1/sessions/${encodeURIComponent(targetSession)}/interactions/commands`,
          { ...action, request_id: request.requestId },
        );
        pendingRef.current = null;
        if (contextRef.current.sessionId === targetSession) {
          await refreshRef.current();
        }
        return true;
      } catch (failure) {
        const status = isAxiosError(failure) ? failure.response?.status : undefined;
        if (status && status < 500) {
          pendingRef.current = null;
        }
        if (contextRef.current.sessionId === targetSession) {
          await refreshRef.current();
          let message = text.failed;
          if (status === 409) {
            message = text.conflict;
          }
          if (status === 422) {
            message = text.invalid;
          }
          setError(message);
        }
        return false;
      } finally {
        busyRef.current = false;
        setBusy(false);
      }
    },
    [sessionId, text.conflict, text.failed, text.invalid],
  );

  const handleMessage = useCallback(
    (message: ServerMessage) => {
      if (!contextRef.current.enabled) {
        return;
      }
      if (
        message.type === "session_interactions_changed" &&
        message.session_id === sessionId &&
        message.version > versionRef.current
      ) {
        void refreshRef.current();
      }
    },
    [sessionId],
  );
  return {
    handleMessage,
    snapshot,
    error,
    busy,
    send,
    refresh: () => refreshRef.current(),
    notice,
    dismissNotice: () => setNotice(null),
  };
}
