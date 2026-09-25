"use client";

import Link from "next/link";
import { CalendarDays, Clock3, LogIn, Play, Radio } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";
import { cn } from "@/lib/utils";

import { roomCopy } from "./room-copy";
import { SessionFiles } from "./session-files";

type Session = components["schemas"]["SessionResponse"];

function SessionTime({ value, locale }: Readonly<{ value: string; locale: Locale }>) {
  const date = new Date(value);
  const language = locale === "fa" ? "fa-IR" : "en-US";
  const day = new Intl.DateTimeFormat(language, {
    day: "numeric",
    month: "short",
  }).format(date);
  const time = new Intl.DateTimeFormat(language, {
    hour: "numeric",
    minute: "2-digit",
  }).format(date);
  return (
    <time dateTime={value} suppressHydrationWarning title={date.toISOString()}>
      {day}
      <br />
      {time}
    </time>
  );
}

export function RoomSessions({
  sessions,
  locale,
}: Readonly<{ sessions: Session[]; locale: Locale }>) {
  const text = roomCopy(locale);
  const sorted = [...sessions].sort((left, right) => {
    if (left.status === "live" && right.status !== "live") {
      return -1;
    }
    if (right.status === "live" && left.status !== "live") {
      return 1;
    }
    return (right.scheduled_start_at ?? right.created_at).localeCompare(
      left.scheduled_start_at ?? left.created_at,
    );
  });
  return (
    <section aria-labelledby="room-sessions" className="space-y-5" data-testid="room-sessions">
      <div className="flex items-end justify-between gap-4">
        <div>
          <p className="mb-1 text-xs font-semibold tracking-wide text-muted-foreground">
            {text.roomActivity}
          </p>
          <h2 className="text-2xl font-semibold tracking-tight" id="room-sessions">
            {text.sessions}
          </h2>
        </div>
        <span className="text-sm tabular-nums text-muted-foreground">{sessions.length}</span>
      </div>
      {sessions.length === 0 ? (
        <div className="rounded-xl border border-dashed bg-card px-6 py-12">
          <CalendarDays aria-hidden="true" className="mb-4 size-6 text-muted-foreground" />
          <p className="font-medium">{text.noSessions}</p>
          <p className="mt-1 text-sm text-muted-foreground">{text.noSessionsHelp}</p>
        </div>
      ) : (
        <ol className="space-y-3">
          {sorted.map((session) => {
            const date = session.scheduled_start_at ?? session.started_at ?? session.created_at;
            const live = session.status === "live";
            const status = text.sessionStatus[session.status];
            return (
              <li
                className={cn(
                  "overflow-hidden rounded-xl border bg-card",
                  live && "border-success/50",
                )}
                data-testid={`room-session-${session.id}`}
                key={session.id}
              >
                <div className="flex flex-col gap-4 p-4 sm:flex-row sm:items-center sm:gap-6 sm:p-6">
                  <div
                    className="flex w-24 shrink-0 flex-col gap-0.5 border-s-2 border-primary/50 ps-3 text-sm font-medium tabular-nums text-foreground [&_time]:flex [&_time]:flex-col"
                    data-testid="session-schedule"
                  >
                    <SessionTime locale={locale} value={date} />
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="mb-1.5 flex flex-wrap items-center gap-2">
                      <span
                        className={cn(
                          "inline-flex items-center gap-1.5 text-xs font-semibold",
                          live ? "text-success" : "text-muted-foreground",
                        )}
                      >
                        {live ? (
                          <Radio aria-hidden="true" className="size-3.5" />
                        ) : (
                          <span aria-hidden="true" className="size-1.5 rounded-full bg-current" />
                        )}
                        {status}
                      </span>
                      {session.scheduled_duration_minutes ? (
                        <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
                          <Clock3 aria-hidden="true" className="size-3.5" />
                          {session.scheduled_duration_minutes} {text.minutes}
                        </span>
                      ) : null}
                    </div>
                    <Link
                      className="block truncate text-lg font-semibold tracking-tight underline-offset-4 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                      href={`/session/${session.id}`}
                    >
                      {session.title}
                    </Link>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {session.scheduled_start_at ? text.scheduledFor : text.createdOn}
                    </p>
                  </div>
                  <TooltipProvider>
                    <div className="flex shrink-0 items-center gap-2 sm:ms-auto">
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <span tabIndex={0}>
                            <Button
                              aria-label={text.playbackUnavailable}
                              disabled
                              size="icon"
                              variant="outline"
                            >
                              <Play aria-hidden="true" className="size-4" />
                            </Button>
                          </span>
                        </TooltipTrigger>
                        <TooltipContent>{text.playbackUnavailable}</TooltipContent>
                      </Tooltip>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button asChild size="icon" variant={live ? "default" : "outline"}>
                            <Link aria-label={text.joinSession} href={`/session/${session.id}`}>
                              <LogIn aria-hidden="true" className="size-4" />
                            </Link>
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>{text.joinSession}</TooltipContent>
                      </Tooltip>
                    </div>
                  </TooltipProvider>
                </div>
                <SessionFiles locale={locale} roomId={session.room_id} sessionId={session.id} />
              </li>
            );
          })}
        </ol>
      )}
    </section>
  );
}
