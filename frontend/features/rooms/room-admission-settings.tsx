"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/button";
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
  const [saved, setSaved] = useState(initialPolicy);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState(false);

  async function save() {
    setPending(true);
    try {
      const { data } = await browserApiClient.put<{ policy: "open" | "lobby" }>(
        `/v1/rooms/${encodeURIComponent(roomId)}/admission`,
        { policy },
      );
      setPolicy(data.policy);
      setSaved(data.policy);
      setError(false);
      router.refresh();
    } catch {
      setError(true);
    } finally {
      setPending(false);
    }
  }

  return (
    <section className="space-y-3" data-testid="room-admission-settings">
      <h2 className="text-lg font-semibold">{text.admissionSettings}</h2>
      <div className="flex flex-wrap items-end gap-3">
        <div className="space-y-1">
          <label className="block text-sm font-medium" htmlFor={`admission-${roomId}`}>
            {text.admissionPolicy}
          </label>
          <select
            className="h-10 rounded-md border border-input bg-background px-3 text-sm focus-visible:ring-2 focus-visible:ring-ring"
            id={`admission-${roomId}`}
            onChange={(event) => setPolicy(event.target.value as "open" | "lobby")}
            value={policy}
          >
            <option value="open">{text.openAdmission}</option>
            <option value="lobby">{text.lobby}</option>
          </select>
        </div>
        <Button disabled={pending || policy === saved} onClick={() => void save()}>
          {text.save}
        </Button>
      </div>
      {error ? (
        <p role="alert" className="text-sm text-destructive">
          {text.admissionSaveFailed}
        </p>
      ) : null}
    </section>
  );
}
