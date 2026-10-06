"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { browserApiClient } from "@/lib/api/browser";
import { messagesFor } from "@/lib/i18n/messages";
import type { Locale } from "@/lib/i18n/config";
import type { Activity, VotersPage } from "./types";

export function ActivityVoters({
  sessionId,
  activity,
  locale,
}: Readonly<{ sessionId: string; activity: Activity; locale: Locale }>) {
  const text = messagesFor(locale).interactions;
  const [open, setOpen] = useState(false);
  const [page, setPage] = useState<VotersPage | null>(null);
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);
  async function load(offset = 0) {
    setBusy(true);
    setFailed(false);
    try {
      const { data } = await browserApiClient.get<VotersPage>(
        `/v1/sessions/${encodeURIComponent(sessionId)}/interactions/activities/${encodeURIComponent(activity.id)}/voters`,
        { params: { offset } },
      );
      setPage((previous) => ({
        ...data,
        voters: offset && previous ? [...previous.voters, ...data.voters] : data.voters,
      }));
    } catch {
      setFailed(true);
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <Button
        size="sm"
        variant="outline"
        onClick={() => {
          setOpen(true);
          void load();
        }}
      >
        {text.viewVoters}
      </Button>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent
          dir={locale === "fa" ? "rtl" : "ltr"}
          className="max-h-[85dvh] overflow-y-auto"
        >
          <DialogHeader>
            <DialogTitle>{text.viewVoters}</DialogTitle>
            <DialogDescription dir="auto" className="break-words">
              {activity.question}
            </DialogDescription>
          </DialogHeader>
          {page ? (
            <ul className="space-y-3">
              {page.voters.map((voter) => (
                <li key={voter.actor_id} className="rounded-lg border p-3">
                  <p className="break-words font-medium" dir="auto">
                    {voter.name}
                  </p>
                  <p className="mt-1 break-words text-sm text-muted-foreground" dir="auto">
                    {voter.selected
                      .map((i) => activity.options[i])
                      .join(locale === "fa" ? "، " : ", ")}
                  </p>
                </li>
              ))}
            </ul>
          ) : null}
          {failed ? (
            <p role="alert" className="text-sm text-destructive">
              {text.failed}
            </p>
          ) : null}
          {busy ? <p role="status">{text.loading}</p> : null}
          {failed ? (
            <Button disabled={busy} onClick={() => void load(page?.next_offset ?? 0)}>
              {text.retry}
            </Button>
          ) : null}
          {!failed && page?.next_offset !== null && page?.next_offset !== undefined ? (
            <Button disabled={busy} onClick={() => void load(page.next_offset ?? 0)}>
              {text.more}
            </Button>
          ) : null}
        </DialogContent>
      </Dialog>
    </>
  );
}
