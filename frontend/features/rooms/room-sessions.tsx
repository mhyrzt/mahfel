"use client";

import Link from "next/link";
import { CalendarDays, Clock3, Film, Radio } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";
import { cn } from "@/lib/utils";

import { roomCopy } from "./room-copy";
import { SessionFiles } from "./session-files";
import { SessionParticipantCount } from "./session-participant-count";
import { SessionSettingsDialog } from "./session-settings-dialog";

type Session = components["schemas"]["SessionResponse"];
type FileRecord = components["schemas"]["FileResponse"];

export interface SessionFileFixture {
  files: FileRecord[];
  usedIds?: string[];
  roomFiles?: FileRecord[];
  attachRoomFile?: (file: FileRecord) => Promise<FileRecord>;
  uploadSessionFile?: (file: File) => Promise<FileRecord>;
}

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
  fileData,
  canManageSession = false,
}: Readonly<{
  sessions: Session[];
  locale: Locale;
  fileData?: Record<string, SessionFileFixture>;
  canManageSession?: boolean;
}>) {
  const text = roomCopy(locale);
  const sorted = [...sessions].sort((left, right) =>
    (left.scheduled_start_at ?? left.created_at).localeCompare(
      right.scheduled_start_at ?? right.created_at,
    ),
  );
  return (
    <section aria-label={text.sessions} className="space-y-5" data-testid="room-sessions">
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
                <div className="grid grid-cols-[5rem_minmax(0,1fr)] items-center gap-x-4 gap-y-3 p-4 sm:grid-cols-[6rem_minmax(0,1fr)_auto] sm:gap-x-6 sm:p-6">
                  <div
                    className="flex min-w-0 flex-col gap-0.5 border-s-2 border-primary/50 ps-3 text-sm font-medium tabular-nums text-foreground [&_time]:flex [&_time]:flex-col"
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
                      <SessionParticipantCount live={live} locale={locale} sessionId={session.id} />
                    </div>
                    <Link
                      className="block truncate text-lg font-semibold tracking-tight underline-offset-4 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                      href={`/session/${session.id}`}
                    >
                      {session.title}
                    </Link>
                  </div>
                  <TooltipProvider>
                    <div className="col-span-2 flex shrink-0 items-center justify-end gap-2 sm:col-span-1 sm:ms-auto">
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <span tabIndex={0}>
                            <Button
                              aria-label={text.playbackUnavailable}
                              disabled
                              size="icon"
                              variant="outline"
                            >
                              <Film aria-hidden="true" className="size-4" />
                            </Button>
                          </span>
                        </TooltipTrigger>
                        <TooltipContent>{text.playbackUnavailable}</TooltipContent>
                      </Tooltip>
                      {canManageSession ? (
                        <SessionSettingsDialog session={session} locale={locale} />
                      ) : null}
                    </div>
                  </TooltipProvider>
                </div>
                <SessionFiles
                  locale={locale}
                  roomId={session.room_id}
                  sessionId={session.id}
                  initialFiles={fileData?.[session.id]?.files}
                  initialUsedIds={fileData?.[session.id]?.usedIds}
                  initialRoomFiles={fileData?.[session.id]?.roomFiles}
                  attachRoomFile={fileData?.[session.id]?.attachRoomFile}
                  uploadSessionFile={fileData?.[session.id]?.uploadSessionFile}
                />
              </li>
            );
          })}
        </ol>
      )}
    </section>
  );
}
