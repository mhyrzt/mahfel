"use client";

import { useEffect, useId, useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";
import { Settings2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";

type Session = components["schemas"]["SessionResponse"];

function localDateTime(value: string | null | undefined): string {
  if (!value) {
    return "";
  }
  const date = new Date(value);
  return new Date(date.getTime() - date.getTimezoneOffset() * 60_000).toISOString().slice(0, 16);
}

export function SessionSettingsDialog({
  session,
  locale,
}: Readonly<{ session: Session; locale: Locale }>) {
  const text = roomCopy(locale);
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState(session.title_overridden ? session.title : "");
  const [start, setStart] = useState("");
  const [duration, setDuration] = useState(session.scheduled_duration_minutes?.toString() ?? "");
  const autoRecordId = useId();
  const [autoRecord, setAutoRecord] = useState(session.auto_record ?? true);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState(false);

  useEffect(() => {
    void Promise.resolve().then(() => setStart(localDateTime(session.scheduled_start_at)));
  }, [session.scheduled_start_at]);

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPending(true);
    setError(false);
    try {
      await browserApiClient.patch(`/v1/sessions/${encodeURIComponent(session.id)}`, {
        ...(session.status === "scheduled" ? { auto_record: autoRecord } : {}),
        title: title.trim() || null,
        scheduled_start_at: start ? new Date(start).toISOString() : null,
        scheduled_duration_minutes: duration ? Number(duration) : null,
      });
      setOpen(false);
      router.refresh();
    } catch {
      setError(true);
    } finally {
      setPending(false);
    }
  }

  return (
    <Dialog onOpenChange={setOpen} open={open}>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <DialogTrigger asChild>
              <Button
                aria-label={`${text.sessionSettings}: ${session.title}`}
                data-testid={`session-settings-${session.id}`}
                size="icon"
                type="button"
                variant="outline"
              >
                <Settings2 aria-hidden="true" className="size-4" />
              </Button>
            </DialogTrigger>
          </TooltipTrigger>
          <TooltipContent>{text.sessionSettings}</TooltipContent>
        </Tooltip>
      </TooltipProvider>
      <DialogContent
        className="max-h-[85dvh] overflow-y-auto"
        data-testid={`session-settings-dialog-${session.id}`}
        dir={locale === "fa" ? "rtl" : "ltr"}
        lang={locale}
      >
        <DialogHeader className="sm:text-start">
          <DialogTitle>{text.sessionSettings}</DialogTitle>
          <DialogDescription>{session.title}</DialogDescription>
        </DialogHeader>
        <form className="space-y-4" onSubmit={(event) => void save(event)}>
          <label className="block space-y-1 text-sm font-medium">
            <span>{text.sessionTitle}</span>
            <Input onChange={(event) => setTitle(event.target.value)} value={title} />
          </label>
          <label className="block space-y-1 text-sm font-medium">
            <span>{text.plannedStart}</span>
            <Input
              onChange={(event) => setStart(event.target.value)}
              type="datetime-local"
              value={start}
            />
          </label>
          <label className="block space-y-1 text-sm font-medium">
            <span>{text.duration}</span>
            <Input
              max={10080}
              min={1}
              onChange={(event) => setDuration(event.target.value)}
              type="number"
              value={duration}
            />
          </label>
          {session.status === "scheduled" ? (
            <div className="flex items-center gap-2 text-sm">
              <Checkbox
                id={autoRecordId}
                checked={autoRecord}
                onCheckedChange={(checked) => setAutoRecord(checked === true)}
              />
              <label htmlFor={autoRecordId}>{text.autoRecord}</label>
            </div>
          ) : null}
          {error ? (
            <p className="text-sm text-destructive" role="alert">
              {text.sessionSettingsFailed}
            </p>
          ) : null}
          <div className="flex justify-end gap-2">
            <Button onClick={() => setOpen(false)} type="button" variant="outline">
              {text.cancel}
            </Button>
            <Button disabled={pending} type="submit">
              {pending ? text.saving : text.save}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
