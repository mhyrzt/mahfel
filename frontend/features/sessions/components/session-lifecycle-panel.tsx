"use client";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { useState } from "react";
import { useRouter } from "next/navigation";
import { CircleStop } from "lucide-react";
import axios from "axios";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
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
  compact = false,
}: Readonly<{ session: Session; canManage: boolean; locale: Locale; compact?: boolean }>) {
  const router = useRouter();
  const text = roomCopy(locale);
  const [pending, setPending] = useState<"start" | "end" | "cancel" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [liveSessionId, setLiveSessionId] = useState<string | null>(null);
  const [confirmingEnd, setConfirmingEnd] = useState(false);

  async function act(action: "start" | "end" | "cancel") {
    setPending(action);
    setError(null);
    setLiveSessionId(null);
    try {
      await browserApiClient.post(`/v1/sessions/${encodeURIComponent(session.id)}/${action}`);
      setConfirmingEnd(false);
      router.refresh();
    } catch (failure) {
      if (action === "start" && axios.isAxiosError(failure) && failure.response?.status === 409) {
        try {
          const { data } = await browserApiClient.get<
            components["schemas"]["Paginated_SessionResponse"]
          >(`/v1/rooms/${encodeURIComponent(session.room_id)}/sessions`, {
            params: { status: "live", per_page: 1 },
          });
          const live = data.data.find((item) => item.status === "live");
          if (live) {
            setLiveSessionId(live.id);
            setError(text.liveSessionConflict);
          } else {
            setError(text.lifecycleFailed);
            router.refresh();
          }
        } catch {
          setError(text.lifecycleFailed);
        }
      } else if (
        action === "start" &&
        axios.isAxiosError(failure) &&
        failure.response?.status === 503
      ) {
        setError(text.recordingStartUnavailable);
      } else {
        setError(text.lifecycleFailed);
      }
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
      className={compact ? "inline-flex" : "space-y-4 rounded-xl border bg-card p-5"}
      data-testid="session-lifecycle"
    >
      <div className="flex flex-wrap items-start justify-between gap-4">
        {!compact ? (
          <div className="space-y-1">
            <h1 className="text-xl font-semibold">{session.title}</h1>
            {status ? <p className="text-sm text-muted-foreground">{status}</p> : null}
          </div>
        ) : null}
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
        {canManage &&
          session.status === "live" &&
          (compact ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  aria-label={text.endSession}
                  className="size-10 rounded-xl text-rose-300 hover:bg-rose-500/15 hover:text-rose-100"
                  disabled={pending !== null}
                  onClick={() => setConfirmingEnd(true)}
                  variant="ghost"
                >
                  <CircleStop className="size-[1.125rem]" aria-hidden="true" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="top">{text.endSession}</TooltipContent>
            </Tooltip>
          ) : (
            <Button
              disabled={pending !== null}
              onClick={() => setConfirmingEnd(true)}
              variant="destructive"
            >
              {pending === "end" ? text.ending : text.endSession}
            </Button>
          ))}
      </div>
      {error && !compact ? (
        <div className="space-y-1 text-sm text-destructive" role="alert">
          <p>{error}</p>
          {liveSessionId ? (
            <Link
              className="font-medium underline underline-offset-2"
              href={`/session/${liveSessionId}`}
            >
              {text.openLiveSession}
            </Link>
          ) : null}
        </div>
      ) : null}
      <Dialog onOpenChange={setConfirmingEnd} open={confirmingEnd}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{text.endSession}</DialogTitle>
            <DialogDescription>{text.confirmEnd}</DialogDescription>
          </DialogHeader>
          {error && compact ? (
            <Alert variant="destructive" className="text-sm text-destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : null}
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
