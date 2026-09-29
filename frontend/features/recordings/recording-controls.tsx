"use client";

import { useEffect, useRef, useState } from "react";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";
import { RecordingControlsView } from "./recording-controls-view";

type Recording = components["schemas"]["RecordingResponse"];
type Action = components["schemas"]["RecordingActionDto"];

export function RecordingControls({
  sessionId,
  locale,
}: Readonly<{ sessionId: string; locale: Locale }>) {
  const [recording, setRecording] = useState<Recording | null>(null);
  const [canControl, setCanControl] = useState(false);
  const [verified, setVerified] = useState(false);
  const [unavailable, setUnavailable] = useState(false);
  const [busy, setBusy] = useState(false);
  const [commandFailed, setCommandFailed] = useState(false);
  const retry = useRef<{ command_id: string; action: Action } | null>(null);
  const path = `/v1/sessions/${encodeURIComponent(sessionId)}`;

  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const abort = new AbortController();
    const refresh = async () => {
      try {
        const [state, capabilities] = await Promise.all([
          browserApiClient.get<Recording[]>(`${path}/recordings`, { signal: abort.signal }),
          browserApiClient
            .get<{ capabilities: string[] }>(`${path}/capabilities`, {
              signal: abort.signal,
            })
            .then((response) => response.data.capabilities.includes("manage_recording"))
            .catch(() => false),
        ]);
        if (active) {
          setRecording((current) => {
            const next = state.data[0] ?? null;
            if (
              current &&
              (!next ||
                (current.id === next.id && next.revision < current.revision) ||
                Date.parse(next.started_at) < Date.parse(current.started_at))
            ) {
              return current;
            }
            return next;
          });
          setCanControl(capabilities);
          setVerified(true);
          setUnavailable(false);
        }
      } catch {
        if (active) {
          setUnavailable(true);
          setVerified(false);
        }
      }
      if (active) {
        timer = setTimeout(() => void refresh(), 3000);
      }
    };
    void refresh();
    return () => {
      active = false;
      abort.abort();
      clearTimeout(timer);
    };
  }, [path]);

  async function command(action: Action) {
    if (busy) {
      return;
    }
    const request =
      retry.current?.action === action
        ? retry.current
        : { command_id: crypto.randomUUID(), action };
    retry.current = request;
    setBusy(true);
    setCommandFailed(false);
    try {
      const response = await browserApiClient.post<Recording>(
        `${path}/recordings/commands`,
        request,
      );
      setRecording(response.data);
      setVerified(true);
      retry.current = null;
    } catch {
      setCommandFailed(true);
    } finally {
      setBusy(false);
    }
  }

  return (
    <RecordingControlsView
      locale={locale}
      recording={recording}
      verified={verified}
      canControl={canControl}
      busy={busy}
      commandFailed={commandFailed}
      unavailable={unavailable}
      onCommand={(action) => void command(action)}
    />
  );
}
