import { AlertCircle, LoaderCircle } from "lucide-react";
import type { ReactNode } from "react";

import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import type { SessionState } from "../session-state";
import { AdmissionQueue } from "./admission-queue";
import { ParticipantRoster } from "./participant-roster";
import { SessionStage } from "./session-stage";
import { SessionSummary } from "./session-summary";
import type {
  AdmissionRequest,
  Participant,
  ParticipantAction,
  ParticipantMediaStreams,
  SessionTiming,
} from "./session-types";
import { WaitingRoomPanel } from "./waiting-room-panel";
import { AccessManagement } from "./access-management";
import { SessionSidebar } from "./session-sidebar";
import { SessionBreadcrumb } from "./session-breadcrumb";
import type { useSessionMedia } from "./use-session-media";
import type { useRemoteSessionMedia } from "./use-remote-session-media";
import { RemoteAudio } from "./remote-audio";

interface SessionScreenProps {
  actorId?: string;
  sessionId?: string;
  roomId?: string;
  roomName?: string;
  activeSurface?: "participants" | "notes" | "whiteboard" | "presentation";
  onSelectSurface?: (surface: "participants" | "notes" | "whiteboard" | "presentation") => void;
  collaboration?: ReactNode;
  files?: ReactNode;
  invites?: ReactNode;
  endAction?: ReactNode;
  recordingControl?: ReactNode;
  title: string;
  state: SessionState;
  admissionRequests?: AdmissionRequest[];
  sessionTiming?: SessionTiming | null;
  locale?: Locale;
  onRetryJoin?: () => void;
  onParticipantAction?: (action: ParticipantAction, participant: Participant) => void;
  actionError?: string | null;
  media?: ReturnType<typeof useSessionMedia>;
  remoteMedia?: ReturnType<typeof useRemoteSessionMedia>;
  canAdmit?: boolean;
  onAdmit?: (request: AdmissionRequest) => void;
  onReject?: (request: AdmissionRequest) => void;
  onLeave?: () => void;
}

export function SessionScreen({
  actorId,
  sessionId,
  roomId,
  roomName,
  activeSurface,
  onSelectSurface,
  collaboration,
  files,
  invites,
  endAction,
  recordingControl,
  title,
  state,
  admissionRequests = [],
  sessionTiming,
  locale = defaultLocale,
  onRetryJoin,
  onParticipantAction,
  actionError,
  media,
  remoteMedia,
  canAdmit,
  onAdmit,
  onReject,
  onLeave,
}: Readonly<SessionScreenProps>) {
  const text = messagesFor(locale).sessions;
  if (state.phase === "loading") {
    return (
      <SessionMessage icon={LoaderCircle} title={text.loadingTitle} message={text.loadingMessage} />
    );
  }

  if (state.phase === "waiting") {
    return (
      <WaitingRoomPanel roomName={state.roomName ?? title} locale={locale} onRetry={onRetryJoin} />
    );
  }

  if (state.phase === "rejected" || state.phase === "unavailable") {
    return (
      <SessionMessage
        icon={AlertCircle}
        title={state.phase === "rejected" ? text.rejectedTitle : text.unavailableTitle}
        message={state.message ?? text.retryMessage}
      />
    );
  }

  const isLive = state.phase === "live";
  const showAdmission = canAdmit ?? state.capabilities.canModerate;
  const participantMedia: Record<string, ParticipantMediaStreams> = {};
  for (const participant of state.participants) {
    const remote = participant.actorId ? remoteMedia?.streams[participant.actorId] : undefined;
    if (remote?.camera || remote?.screen) {
      participantMedia[participant.id] = {
        camera: remote.camera,
        screenShare: remote.screen,
      };
    }
  }
  if (state.participantId) {
    participantMedia[state.participantId] = {
      camera: media?.streams.camera,
      screenShare: media?.streams.screen,
    };
  }
  const displayParticipants = state.participants.map((participant) => {
    let microphoneActive = !participant.muted;
    if (participant.id === state.participantId && media) {
      microphoneActive = Boolean(media.enabled.microphone);
    } else if (participant.id !== state.participantId && remoteMedia) {
      microphoneActive = Boolean(
        participant.actorId && remoteMedia.streams[participant.actorId]?.microphone,
      );
    }
    const muted = Boolean(participant.muted) || !microphoneActive;
    return { ...participant, muted, speaking: Boolean(participant.speaking) && !muted };
  });
  return (
    <SessionSidebar
      locale={locale}
      showAdmission={showAdmission}
      participants={
        <ParticipantRoster
          participants={displayParticipants}
          summary={sessionTiming ? <SessionSummary timing={sessionTiming} locale={locale} /> : null}
          locale={locale}
        />
      }
      admission={
        <AdmissionQueue
          requests={admissionRequests}
          locale={locale}
          onAdmit={onAdmit}
          onReject={onReject}
        />
      }
      access={
        sessionId ? (
          <AccessManagement actorId={actorId} sessionId={sessionId} locale={locale} />
        ) : null
      }
      files={files}
      invites={invites}
    >
      <div
        dir={locale === "fa" ? "rtl" : "ltr"}
        className="flex h-full min-h-0 min-w-0 flex-col overflow-hidden"
      >
        {roomId ? (
          <SessionBreadcrumb
            roomId={roomId}
            roomName={roomName ?? title}
            title={title}
            locale={locale}
          />
        ) : null}
        {!isLive ? (
          <header className="rounded-xl border bg-card px-4 py-3 md:px-6">
            <h1 className="truncate text-lg font-semibold">{title}</h1>
          </header>
        ) : null}
        {actionError ? (
          <p
            className="rounded-lg border border-destructive/40 p-3 text-sm text-destructive"
            role="alert"
          >
            {actionError}
          </p>
        ) : null}
        {media?.error ? (
          <p
            className="rounded-lg border border-destructive/40 p-3 text-sm text-destructive"
            role="alert"
          >
            {media.error}
          </p>
        ) : null}
        {remoteMedia?.error ? (
          <p
            className="rounded-lg border border-destructive/40 p-3 text-sm text-destructive"
            role="alert"
          >
            {remoteMedia.error}
          </p>
        ) : null}
        {Object.entries(remoteMedia?.streams ?? {}).map(([owner, sources]) =>
          sources.microphone ? (
            <RemoteAudio key={owner} locale={locale} stream={sources.microphone} />
          ) : null,
        )}
        <SessionStage
          currentActorId={actorId}
          currentParticipantId={state.participantId}
          activeSurface={activeSurface}
          onSelectSurface={onSelectSurface}
          collaboration={collaboration}
          participantActions={
            isLive
              ? [
                  ...(state.capabilities.canMuteOthers ? ["mute" as const] : []),
                  ...(state.capabilities.canRemoveParticipant
                    ? ["remove" as const, "block" as const]
                    : []),
                ]
              : []
          }
          participants={displayParticipants}
          participantMedia={participantMedia}
          cameraEnabled={media?.enabled.camera}
          microphoneEnabled={media?.enabled.microphone}
          screenShareEnabled={media?.enabled.screen}
          mediaBusy={media?.busy}
          onToggleMicrophone={media ? () => void media.toggle("microphone") : undefined}
          onToggleCamera={media ? () => void media.toggle("camera") : undefined}
          onToggleScreenShare={media ? () => void media.toggle("screen") : undefined}
          status={isLive ? "live" : "ended"}
          title={title}
          toolbarCapabilities={isLive ? state.capabilities : undefined}
          locale={locale}
          onParticipantAction={onParticipantAction}
          onLeave={onLeave}
          endAction={endAction}
          recordingControl={isLive ? recordingControl : undefined}
        />
      </div>
    </SessionSidebar>
  );
}

function SessionMessage({
  icon: Icon,
  title,
  message,
}: Readonly<{ icon: typeof AlertCircle; title: string; message: string }>) {
  return (
    <Card className="mx-auto max-w-lg">
      <CardHeader className="items-center text-center">
        <Icon className="size-6 text-muted-foreground" aria-hidden="true" />
        <CardTitle>{title}</CardTitle>
        <CardDescription>{message}</CardDescription>
      </CardHeader>
    </Card>
  );
}
