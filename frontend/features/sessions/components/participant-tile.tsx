"use client";

import { useEffect, useRef } from "react";
import { Hand, Camera, MicOff, MonitorUp } from "lucide-react";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import { ParticipantActionsMenu } from "./participant-actions-menu";
import type { Participant, ParticipantAction } from "./session-types";

interface ParticipantTileProps {
  actions?: ParticipantAction[];
  cameraStream?: MediaStream;
  participant: Participant;
  screenShareStream?: MediaStream;
  locale?: Locale;
  onAction?: (action: ParticipantAction, participant: Participant) => void;
}

export function ParticipantTile({
  actions = [],
  cameraStream,
  participant,
  screenShareStream,
  locale = defaultLocale,
  onAction,
}: Readonly<ParticipantTileProps>) {
  const text = messagesFor(locale).sessions;
  const cameraEnabled = participant.videoEnabled === true;
  const screenShareEnabled = participant.screenShareEnabled === true;

  if (screenShareEnabled) {
    return (
      <article
        aria-label={`${participant.name}: ${text.screenSharePreview}`}
        className={`session-participant-enter relative aspect-video w-full overflow-hidden rounded-xl bg-slate-950 ring-1 ring-white/10 sm:w-[min(100%,var(--stage-tile-max-width))] sm:justify-self-center ${
          participant.speaking ? "ring-2 ring-success ring-offset-2" : ""
        }`}
      >
        {screenShareStream ? (
          <StreamVideo
            className="absolute inset-0 size-full object-contain"
            label={`${participant.name}: ${text.screenSharePreview}`}
            stream={screenShareStream}
          />
        ) : (
          <ScreenSharePlaceholder label={text.screenSharePreview} />
        )}
        <div className="absolute inset-x-0 bottom-0 flex items-end justify-between gap-3 bg-gradient-to-t from-black/80 via-black/35 to-transparent p-3 pt-10 sm:p-4 sm:pt-12">
          <p className="truncate text-sm font-medium text-white">
            {participant.name}
            {participant.handRaised ? (
              <Hand
                className="ms-2 inline size-4 text-warning"
                aria-label={messagesFor(locale).interactions.handRaised}
              />
            ) : null}
          </p>
          <ParticipantActionsMenu
            actions={actions}
            participant={participant}
            locale={locale}
            tone="dark"
            onAction={onAction}
          />
        </div>
        <div className="absolute right-3 top-3 size-20 overflow-hidden rounded-full border-2 border-white/90 bg-slate-900 shadow-lg sm:right-4 sm:top-4 sm:size-28">
          {cameraEnabled && cameraStream ? (
            <StreamVideo
              className="size-full object-cover"
              label={`${participant.name}: ${text.cameraActive}`}
              stream={cameraStream}
            />
          ) : (
            <ParticipantAvatar participant={participant} />
          )}
          {cameraEnabled && !cameraStream ? (
            <Camera
              aria-label={text.cameraActive}
              className="absolute bottom-1 right-1 size-5 rounded-full bg-slate-800 p-1 text-white"
            />
          ) : null}
        </div>
      </article>
    );
  }

  return (
    <article
      aria-label={participant.name}
      className="session-participant-enter flex min-h-0 w-full flex-col items-center justify-center gap-2 bg-transparent sm:gap-3"
    >
      <div
        className={`group relative aspect-square w-[min(24vw,var(--stage-mobile-avatar-size,6rem))] shrink-0 rounded-full bg-slate-900 ring-2 ring-white/15 sm:w-[min(14vw,var(--stage-avatar-size,10rem))] sm:ring-4 ${
          participant.speaking
            ? "speaking-avatar ring-success ring-offset-2 ring-offset-background"
            : ""
        }`}
      >
        {cameraEnabled && cameraStream ? (
          <StreamVideo
            className="size-full rounded-full object-cover"
            label={`${participant.name}: ${text.cameraActive}`}
            stream={cameraStream}
          />
        ) : (
          <ParticipantAvatar participant={participant} />
        )}
        {participant.muted ? (
          <span
            aria-label={text.muted}
            className="absolute -bottom-3.5 left-1/2 flex size-7 -translate-x-1/2 items-center justify-center rounded-full border-2 border-background bg-destructive text-white shadow-sm"
            role="img"
          >
            <MicOff aria-hidden="true" className="size-3.5" />
          </span>
        ) : null}
        {actions.length > 0 ? (
          <div className="absolute -top-4 left-1/2 z-10 -translate-x-1/2 opacity-100 transition-opacity [@media(hover:hover)]:pointer-events-none [@media(hover:hover)]:opacity-0 [@media(hover:hover)]:group-hover:pointer-events-auto [@media(hover:hover)]:group-hover:opacity-100 [@media(hover:hover)]:group-focus-within:pointer-events-auto [@media(hover:hover)]:group-focus-within:opacity-100">
            <ParticipantActionsMenu
              actions={actions}
              participant={participant}
              locale={locale}
              compact
              icon="settings"
              onAction={onAction}
            />
          </div>
        ) : null}
      </div>
      <div className="flex min-w-0 max-w-full items-center justify-center gap-1 px-2 text-center">
        {participant.handRaised ? (
          <Hand
            className="size-4 shrink-0 text-warning"
            aria-label={messagesFor(locale).interactions.handRaised}
          />
        ) : null}
        <p className="max-w-[min(60vw,16rem)] truncate text-sm font-medium text-foreground">
          {participant.name}
        </p>
      </div>
    </article>
  );
}

function ParticipantAvatar({ participant }: Readonly<{ participant: Participant }>) {
  return (
    <Avatar className="size-full rounded-full">
      {participant.avatarUrl ? (
        <AvatarImage alt={participant.name} src={participant.avatarUrl} />
      ) : null}
      <AvatarFallback className="rounded-full bg-slate-800 text-2xl font-semibold text-white sm:text-4xl">
        {participant.initials}
      </AvatarFallback>
    </Avatar>
  );
}

function ScreenSharePlaceholder({ label }: Readonly<{ label: string }>) {
  return (
    <div
      aria-label={label}
      className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-[radial-gradient(ellipse_at_top,_rgba(59,130,246,0.22),_transparent_55%),linear-gradient(145deg,_#172033,_#080c14_70%)] text-white/75"
      role="img"
    >
      <div className="flex aspect-video w-3/5 items-center justify-center rounded-lg border border-white/15 bg-white/5 shadow-2xl">
        <MonitorUp aria-hidden="true" className="size-8" strokeWidth={1.5} />
      </div>
      <span className="text-xs font-medium sm:text-sm">{label}</span>
    </div>
  );
}

function StreamVideo({
  className,
  label,
  stream,
}: Readonly<{ className: string; label: string; stream: MediaStream }>) {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    const video = videoRef.current;
    if (video === null) {
      return () => undefined;
    }

    video.srcObject = stream;
    void video.play().catch(() => undefined);

    return () => {
      if (video.srcObject === stream) {
        video.srcObject = null;
      }
    };
  }, [stream]);

  return (
    <video aria-label={label} autoPlay className={className} muted playsInline ref={videoRef} />
  );
}
