"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";

export function RoomAdmissionSettings({
  roomId,
  initialPolicy,
  locale,
}: Readonly<{ roomId: string; initialPolicy: "open" | "lobby"; locale: Locale }>) {
  const text = roomCopy(locale);
  const router = useRouter();
  const [policy, setPolicy] = useState(initialPolicy);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState(false);

  async function toggle() {
    const next = policy === "lobby" ? "open" : "lobby";
    setPending(true);
    try {
      const { data } = await browserApiClient.put<{ policy: "open" | "lobby" }>(
        `/v1/rooms/${encodeURIComponent(roomId)}/admission`,
        { policy: next },
      );
      setPolicy(data.policy);
      setError(false);
      router.refresh();
    } catch {
      setError(true);
    } finally {
      setPending(false);
    }
  }

  return (
    <section className="space-y-3 border-b pb-5" data-testid="room-admission-settings">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h2 className="text-sm font-semibold">{text.admissionPolicy}</h2>
          <p className="text-xs text-muted-foreground">
            {policy === "lobby" ? text.lobby : text.openAdmission}
          </p>
        </div>
        <button
          aria-checked={policy === "lobby"}
          aria-label={text.lobby}
          className="relative h-6 w-11 shrink-0 rounded-full bg-muted-foreground/40 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring data-[checked=true]:bg-primary disabled:opacity-50"
          data-checked={policy === "lobby"}
          data-testid="room-admission-toggle"
          disabled={pending}
          onClick={() => void toggle()}
          role="switch"
          type="button"
        >
          <span
            aria-hidden="true"
            className="absolute start-0.5 top-0.5 size-5 rounded-full bg-background shadow-sm transition-transform data-[checked=true]:translate-x-5 rtl:data-[checked=true]:-translate-x-5"
            data-checked={policy === "lobby"}
          />
        </button>
      </div>
      {error ? (
        <p role="alert" className="text-sm text-destructive">
          {text.admissionSaveFailed}
        </p>
      ) : null}
    </section>
  );
}
