"use client";

import type { CSSProperties } from "react";
import { CalendarClock } from "lucide-react";

import { Card, CardContent } from "@/components/ui/card";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { formatMessage, messagesFor } from "@/lib/i18n/messages";

import { ParticipantTile } from "./participant-tile";
import { SessionStatusBadge } from "./session-status-badge";
import { SessionToolbar, type SessionToolbarCapabilities } from "./session-toolbar";
import type {
  Participant,
  ParticipantAction,
  ParticipantMediaStreams,
  SessionStatus,
} from "./session-types";

interface SessionStageProps {
  participantActions?: ParticipantAction[];
  title: string;
  status: SessionStatus;
  participants: Participant[];
  scheduledFor?: string;
  toolbarCapabilities?: SessionToolbarCapabilities;
  participantMedia?: Readonly<Record<string, ParticipantMediaStreams>>;
  locale?: Locale;
}

export function SessionStage({
  title,
  status,
  participants,
  participantActions = [],
  scheduledFor,
  toolbarCapabilities,
  participantMedia,
  locale = defaultLocale,
}: Readonly<SessionStageProps>) {
  const text = messagesFor(locale).sessions;
  const participantCount = Math.max(participants.length, 1);
  const desktopColumns =
    participantCount <= 2 ? participantCount : Math.min(4, Math.ceil(participantCount / 2));
  const desktopRows = Math.ceil(participantCount / desktopColumns);
  if (status === "scheduled") {
    return (
      <Card>
        <CardContent className="flex aspect-video flex-col items-center justify-center gap-4 text-center">
          <div className="rounded-full bg-muted p-3">
            <CalendarClock className="size-6 text-muted-foreground" aria-hidden="true" />
          </div>
          <div className="space-y-1">
            <p className="font-semibold">{title}</p>
            <p className="text-sm text-muted-foreground">
              {scheduledFor
                ? formatMessage(text.scheduledFor, { time: scheduledFor })
                : text.schedulingDetailsMissing}
            </p>
          </div>
          <SessionStatusBadge status={status} locale={locale} />
        </CardContent>
      </Card>
    );
  }

  if (status === "ended") {
    return (
      <Card>
        <CardContent className="flex aspect-video flex-col items-center justify-center gap-3 text-center">
          <SessionStatusBadge status={status} locale={locale} />
          <div className="space-y-1">
            <p className="font-semibold">{title}</p>
            <p className="text-sm text-muted-foreground">{text.endedDescription}</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <section
      className="flex min-h-[calc(100svh-9rem)] flex-col gap-3 sm:gap-4 lg:min-h-[calc(100svh-8rem)]"
      aria-label={formatMessage(text.stageLabel, { title })}
    >
      <h1 className="shrink-0 text-center text-xl font-semibold tracking-tight">{title}</h1>
      <div
        className={`grid flex-1 content-center gap-2 sm:min-h-[calc(100svh-13rem)] sm:grid-cols-[repeat(var(--stage-grid-columns),minmax(0,1fr))] sm:gap-3 ${participants.length === 1 ? "grid-cols-1" : "grid-cols-2"}`}
        style={
          {
            "--stage-grid-columns": desktopColumns,
            "--stage-grid-rows": desktopRows,
            "--stage-tile-max-width": `calc((100svh - 13rem) * 16 / 9 / ${desktopRows})`,
          } as CSSProperties
        }
      >
        {participants.map((participant) => (
          <ParticipantTile
            actions={participantActions}
            cameraStream={participantMedia?.[participant.id]?.camera}
            key={participant.id}
            participant={participant}
            screenShareStream={participantMedia?.[participant.id]?.screenShare}
            locale={locale}
          />
        ))}
      </div>
      <div className="fixed inset-x-0 bottom-[calc(4rem+env(safe-area-inset-bottom))] z-40 flex justify-center px-4 md:bottom-4 md:pb-0">
        <SessionToolbar {...toolbarCapabilities} locale={locale} />
      </div>
    </section>
  );
}
