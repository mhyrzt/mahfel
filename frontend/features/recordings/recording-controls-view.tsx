"use client";

import {
  Circle,
  CircleCheck,
  CircleAlert,
  LoaderCircle,
  Pause,
  Play,
  Square,
  TriangleAlert,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";
import { recordingCopy } from "./recording-copy";

type Recording = Pick<components["schemas"]["RecordingResponse"], "status" | "warning" | "partial">;
type Action = components["schemas"]["RecordingActionDto"];

export function RecordingControlsView({
  locale,
  recording,
  verified,
  canControl,
  busy = false,
  commandFailed = false,
  unavailable = false,
  onCommand,
}: Readonly<{
  locale: Locale;
  recording: Recording | null;
  verified: boolean;
  canControl: boolean;
  busy?: boolean;
  commandFailed?: boolean;
  unavailable?: boolean;
  onCommand: (action: Action) => void;
}>) {
  const text = recordingCopy(locale);
  const status = recording?.status ?? "off";
  const active = status === "starting" || status === "recording";
  const unfinished = active || status === "paused" || status === "stopping";
  const labels: Record<string, string> = {
    off: text.off,
    starting: text.starting,
    recording: text.recording,
    paused: text.paused,
    stopping: text.stopping,
    finalizing: text.finalizing,
    ready: text.ready,
    failed: text.failedStatus,
  };
  const unknownText = unavailable ? text.unavailable : text.loading;
  const knownText = busy ? text.updating : (labels[status] ?? text.unavailable);
  const statusText = verified ? knownText : unknownText;
  const warning = Boolean(recording?.warning || recording?.partial || commandFailed || unavailable);
  const pending =
    busy || (!verified && !unavailable) || ["starting", "stopping", "finalizing"].includes(status);
  let Icon = Circle;
  if (pending) {
    Icon = LoaderCircle;
  } else if (unavailable || status === "failed") {
    Icon = CircleAlert;
  } else if (status === "paused") {
    Icon = Pause;
  } else if (status === "ready") {
    Icon = CircleCheck;
  }
  const markerClass =
    verified && active && !pending
      ? "fill-rose-400 text-rose-400 motion-safe:animate-pulse"
      : "text-slate-400";
  const actions: Action[] = [];
  if (!unfinished) {
    actions.push("start");
  }
  if (active) {
    actions.push("pause");
  }
  if (status === "paused") {
    actions.push("resume");
  }
  if (unfinished && status !== "stopping") {
    actions.push("stop");
  }
  const controls = {
    start: { label: text.start, icon: Circle },
    pause: { label: text.pause, icon: Pause },
    resume: { label: text.resume, icon: Play },
    stop: { label: text.stop, icon: Square },
  };

  return (
    <div
      className="shrink-0"
      dir={locale === "fa" ? "rtl" : "ltr"}
      lang={locale}
      data-recording-control
    >
      <DropdownMenu modal={false} dir={locale === "fa" ? "rtl" : "ltr"}>
        <DropdownMenuTrigger asChild>
          <Button
            aria-label={`${text.label}: ${statusText}`}
            title={`${text.label}: ${statusText}`}
            variant="ghost"
            size="icon"
            className="relative size-10 rounded-xl text-slate-400 transition-colors hover:bg-slate-800 hover:text-white data-[state=open]:bg-slate-800 data-[state=open]:text-white"
          >
            <Icon
              aria-hidden="true"
              className={`size-4 shrink-0 ${markerClass} ${pending ? "motion-safe:animate-spin" : ""}`}
              strokeWidth={1.8}
            />
            {warning ? (
              <TriangleAlert
                aria-hidden="true"
                className="absolute end-1 top-1 size-2.5 text-amber-400"
              />
            ) : null}
            <span role="status" className="sr-only">
              {statusText}
              {recording?.partial ? ` · ${text.partial}` : ""}
            </span>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="start"
          side="top"
          sideOffset={12}
          collisionPadding={16}
          lang={locale}
          aria-label={text.label}
          className="w-72 max-w-[calc(100vw-2rem)] rounded-xl border-slate-800 bg-slate-950 p-2 text-slate-100 shadow-xl"
        >
          <DropdownMenuLabel className="px-3 pb-1 pt-2 text-xs font-normal text-slate-400">
            {text.label}
          </DropdownMenuLabel>
          <div className="space-y-2 px-3 pb-3">
            <p className="text-base font-semibold tracking-tight">{statusText}</p>
            <p className="text-xs leading-relaxed text-slate-400">{text.scope}</p>
            {recording?.partial ? <p className="text-xs text-amber-300">{text.partial}</p> : null}
            {recording?.warning ? (
              <p className="text-xs leading-relaxed text-amber-300">{text.failed}</p>
            ) : null}
            {commandFailed ? (
              <p className="text-xs leading-relaxed text-amber-300">{text.commandFailed}</p>
            ) : null}
            {unavailable ? (
              <p className="text-xs leading-relaxed text-amber-300">{text.unavailable}</p>
            ) : null}
          </div>
          {canControl && verified ? (
            <>
              <DropdownMenuSeparator className="mx-1 bg-slate-800" />
              {actions.map((action) => {
                const ActionIcon = controls[action].icon;
                return (
                  <DropdownMenuItem
                    key={action}
                    disabled={busy}
                    className={`gap-3 rounded-lg px-3 py-2.5 ${action === "stop" ? "text-rose-300 focus:bg-rose-500/10 focus:text-rose-200" : "focus:bg-slate-800 focus:text-white"}`}
                    onSelect={(event) => {
                      event.preventDefault();
                      onCommand(action);
                    }}
                  >
                    <ActionIcon
                      aria-hidden="true"
                      className={action === "start" ? "fill-rose-400 text-rose-400" : undefined}
                    />
                    {controls[action].label}
                  </DropdownMenuItem>
                );
              })}
            </>
          ) : (
            <p className="border-t border-slate-800 px-3 py-2 text-xs text-slate-500">
              {text.readOnly}
            </p>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
      {recording?.warning ? (
        <p role="alert" className="sr-only">
          {text.failed}
        </p>
      ) : null}
      {commandFailed ? (
        <p role="alert" className="sr-only">
          {text.commandFailed}
        </p>
      ) : null}
      {unavailable ? (
        <p role="alert" className="sr-only">
          {text.unavailable}
        </p>
      ) : null}
    </div>
  );
}
