"use client";

import type { CSSProperties, ReactNode } from "react";
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
  currentActorId?: string;
  currentParticipantId?: string | null;
  activeSurface?: "participants" | "notes" | "whiteboard" | "presentation";
  onSelectSurface?: (surface: "participants" | "notes" | "whiteboard" | "presentation") => void;
  collaboration?: ReactNode;
  participantActions?: ParticipantAction[];
  title: string;
  status: SessionStatus;
  participants: Participant[];
  scheduledFor?: string;
  toolbarCapabilities?: SessionToolbarCapabilities;
  participantMedia?: Readonly<Record<string, ParticipantMediaStreams>>;
  cameraEnabled?: boolean;
  microphoneEnabled?: boolean;
  screenShareEnabled?: boolean;
  mediaBusy?: boolean;
  onToggleMicrophone?: () => void;
  onToggleCamera?: () => void;
  onToggleScreenShare?: () => void;
  locale?: Locale;
  onParticipantAction?: (action: ParticipantAction, participant: Participant) => void;
  onLeave?: () => void;
  endAction?: ReactNode;
}

export function SessionStage({
  currentActorId,
  currentParticipantId,
  activeSurface = "participants",
  onSelectSurface,
  collaboration,
  title,
  status,
  participants,
  participantActions = [],
  scheduledFor,
  toolbarCapabilities,
  participantMedia,
  cameraEnabled,
  microphoneEnabled,
  screenShareEnabled,
  mediaBusy,
  onToggleMicrophone,
  onToggleCamera,
  onToggleScreenShare,
  locale = defaultLocale,
  onParticipantAction,
  onLeave,
  endAction,
}: Readonly<SessionStageProps>) {
  const text = messagesFor(locale).sessions;
  const participantCount = Math.max(participants.length, 1);
  const desktopColumns =
    participantCount <= 2 ? participantCount : Math.min(4, Math.ceil(participantCount / 2));
  const desktopRows = Math.ceil(participantCount / desktopColumns);
  const mobileRows = Math.ceil(participantCount / (participantCount === 1 ? 1 : 2));
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
      className="session-stage-enter flex min-h-0 flex-1 flex-col gap-3 overflow-hidden pb-16 pt-4 sm:gap-4"
      aria-label={formatMessage(text.stageLabel, { title })}
    >
      {activeSurface === "participants" ? (
        <div
          className={`session-participant-grid grid min-h-0 flex-1 content-center gap-2 sm:grid-cols-[repeat(var(--stage-grid-columns),minmax(0,1fr))] sm:gap-3 ${participants.length === 1 ? "grid-cols-1" : "grid-cols-2"}`}
          style={
            {
              "--stage-grid-columns": desktopColumns,
              "--stage-grid-rows": desktopRows,
              "--stage-mobile-rows": mobileRows,
              "--stage-tile-max-width": `calc((100svh - 13rem) * 16 / 9 / ${desktopRows})`,
              "--stage-avatar-size": `clamp(2.75rem, calc((100svh - 13rem) / ${desktopRows} - 2.75rem), 10rem)`,
              "--stage-mobile-avatar-size": `clamp(1.75rem, calc((100svh - 11rem) / ${mobileRows} - 2rem), 6rem)`,
            } as CSSProperties
          }
        >
          {participants.map((participant) => (
            <ParticipantTile
              actions={
                participant.id === currentParticipantId ||
                (participant.actorId && participant.actorId === currentActorId)
                  ? []
                  : participantActions
              }
              cameraStream={participantMedia?.[participant.id]?.camera}
              key={participant.id}
              participant={
                participant.id === currentParticipantId
                  ? { ...participant, videoEnabled: cameraEnabled, screenShareEnabled }
                  : {
                      ...participant,
                      videoEnabled: Boolean(participantMedia?.[participant.id]?.camera),
                      screenShareEnabled: Boolean(participantMedia?.[participant.id]?.screenShare),
                    }
              }
              screenShareStream={participantMedia?.[participant.id]?.screenShare}
              locale={locale}
              onAction={onParticipantAction}
            />
          ))}
        </div>
      ) : (
        <div className="session-surface-enter min-h-0 flex-1 overflow-hidden" key={activeSurface}>
          {collaboration}
        </div>
      )}
      <div className="fixed inset-x-0 bottom-[calc(4rem+env(safe-area-inset-bottom))] z-40 flex justify-center px-4 md:bottom-4 md:pb-0">
        <SessionToolbar
          {...toolbarCapabilities}
          activeSurface={activeSurface}
          onSelectSurface={onSelectSurface}
          locale={locale}
          onLeave={onLeave}
          endAction={endAction}
          cameraEnabled={cameraEnabled}
          microphoneEnabled={microphoneEnabled}
          screenShareEnabled={screenShareEnabled}
          mediaBusy={mediaBusy}
          onToggleMicrophone={onToggleMicrophone}
          onToggleCamera={onToggleCamera}
          onToggleScreenShare={onToggleScreenShare}
        />
      </div>
    </section>
  );
}
