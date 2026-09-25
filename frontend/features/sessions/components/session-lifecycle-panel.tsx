"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { roomCopy } from "@/features/rooms/room-copy";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

type Session = components["schemas"]["SessionResponse"];

export function SessionLifecyclePanel({
  session,
  canManage,
  locale,
}: Readonly<{ session: Session; canManage: boolean; locale: Locale }>) {
  const router = useRouter();
  const text = roomCopy(locale);
  const [pending, setPending] = useState<"start" | "end" | "cancel" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [confirmingEnd, setConfirmingEnd] = useState(false);

  async function act(action: "start" | "end" | "cancel") {
    setPending(action);
    setError(null);
    try {
      await browserApiClient.post(`/v1/sessions/${encodeURIComponent(session.id)}/${action}`);
      setConfirmingEnd(false);
      router.refresh();
    } catch {
      setError(text.lifecycleFailed);
    } finally {
      setPending(null);
    }
  }

  let status;
  if (session.status === "scheduled") {
    status = text.scheduledSession;
  } else if (session.status === "ended") {
    status = text.endedSession;
  } else if (session.status === "cancelled") {
    status = text.cancelledSession;
  }

  return (
    <section
      aria-label={session.title}
      className="space-y-4 rounded-xl border bg-card p-5"
      data-testid="session-lifecycle"
    >
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="space-y-1">
          <h1 className="text-xl font-semibold">{session.title}</h1>
          {status ? <p className="text-sm text-muted-foreground">{status}</p> : null}
        </div>
        {canManage && session.status === "scheduled" ? (
          <div className="flex flex-wrap gap-2">
            <Button disabled={pending !== null} onClick={() => void act("start")}>
              {pending === "start" ? text.starting : text.startSession}
            </Button>
            <Button
              disabled={pending !== null}
              onClick={() => void act("cancel")}
              variant="outline"
            >
              {pending === "cancel" ? text.cancelling : text.cancelSession}
            </Button>
          </div>
        ) : null}
        {canManage && session.status === "live" ? (
          <Button
            disabled={pending !== null}
            onClick={() => setConfirmingEnd(true)}
            variant="destructive"
          >
            {pending === "end" ? text.ending : text.endSession}
          </Button>
        ) : null}
      </div>
      {error ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}
      <Dialog onOpenChange={setConfirmingEnd} open={confirmingEnd}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{text.endSession}</DialogTitle>
            <DialogDescription>{text.confirmEnd}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button onClick={() => setConfirmingEnd(false)} variant="outline">
              {text.keepSession}
            </Button>
            <Button
              disabled={pending !== null}
              onClick={() => void act("end")}
              variant="destructive"
            >
              {pending === "end" ? text.ending : text.endSession}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </section>
  );
}
