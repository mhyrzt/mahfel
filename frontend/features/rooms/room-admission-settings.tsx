"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";

import { Switch } from "@/components/ui/switch";
import { Alert, AlertDescription } from "@/components/ui/alert";

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
        <Switch
          checked={policy === "lobby"}
          aria-label={text.lobby}
          data-testid="room-admission-toggle"
          disabled={pending}
          onCheckedChange={() => void toggle()}
        />
      </div>
      {error ? (
        <Alert variant="destructive">
          <AlertDescription>{text.admissionSaveFailed}</AlertDescription>
        </Alert>
      ) : null}
    </section>
  );
}
