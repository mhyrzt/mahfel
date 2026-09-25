"use client";

import { useCallback, useEffect, useState } from "react";

import { AdmissionQueue } from "@/features/sessions/components/admission-queue";
import type { AdmissionRequest } from "@/features/sessions/components/session-types";
import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";

interface RoomLobbyEntry {
  request_id: string;
  actor_id: string;
  requested_at: string;
}

export function RoomLobby({ roomId, locale }: Readonly<{ roomId: string; locale: Locale }>) {
  const text = roomCopy(locale);
  const [requests, setRequests] = useState<AdmissionRequest[]>([]);
  const [error, setError] = useState(false);
  const path = `/v1/rooms/${encodeURIComponent(roomId)}/lobby`;

  const refresh = useCallback(async () => {
    try {
      const { data } = await browserApiClient.get<{ entries: RoomLobbyEntry[] }>(path);
      setRequests(
        data.entries.map((entry) => ({
          id: entry.request_id,
          actorId: entry.actor_id,
          name: `${text.participant} ${entry.actor_id.slice(0, 4)}`,
          initials: "P",
          requestedAt: entry.requested_at,
        })),
      );
      setError(false);
    } catch {
      setError(true);
    }
  }, [path, text.participant]);

  useEffect(() => {
    void Promise.resolve().then(refresh);
    const interval = window.setInterval(() => void refresh(), 3_000);
    return () => window.clearInterval(interval);
  }, [refresh]);

  async function decide(request: AdmissionRequest, decision: "admit" | "reject") {
    if (!request.actorId) {
      return;
    }
    try {
      await browserApiClient.post(`${path}/${encodeURIComponent(request.actorId)}/${decision}`);
      setRequests((previous) => previous.filter((item) => item.id !== request.id));
      setError(false);
    } catch {
      setError(true);
    }
  }

  return (
    <section aria-label={text.roomRequests} data-testid="room-lobby" className="space-y-3">
      <h2 className="text-lg font-semibold">{text.roomRequests}</h2>
      {error ? (
        <p role="alert" className="text-sm text-destructive">
          {text.lobbyUnavailable}
        </p>
      ) : null}
      <AdmissionQueue
        requests={requests}
        locale={locale}
        onAdmit={(request) => void decide(request, "admit")}
        onReject={(request) => void decide(request, "reject")}
      />
    </section>
  );
}
