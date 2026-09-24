import { AlertCircle, LoaderCircle } from "lucide-react";

import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import type { SessionState } from "../session-state";
import { AdmissionQueue } from "./admission-queue";
import { ParticipantRoster } from "./participant-roster";
import { SessionStage } from "./session-stage";
import { SessionSummary } from "./session-summary";
import type { AdmissionRequest } from "./session-types";
import { WaitingRoomPanel } from "./waiting-room-panel";

interface SessionScreenProps {
  title: string;
  state: SessionState;
  admissionRequests?: AdmissionRequest[];
  locale?: Locale;
  onRetryJoin?: () => void;
}

export function SessionScreen({
  title,
  state,
  admissionRequests = [],
  locale = defaultLocale,
  onRetryJoin,
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

  if (state.phase === "ended") {
    return (
      <SessionStage
        title={title}
        status="ended"
        participants={state.participants}
        locale={locale}
      />
    );
  }

  return (
    <div className="space-y-6">
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_20rem]">
        <SessionStage
          participantActions={state.capabilities.canModerate ? ["mute", "remove", "block"] : []}
          participants={state.participants}
          status="live"
          title={title}
          toolbarCapabilities={state.capabilities}
          locale={locale}
        />
        <ParticipantRoster participants={state.participants} locale={locale} />
      </div>
      <SessionSummary
        duration={text.inProgress}
        participantCount={state.participants.length}
        startedAt={text.liveNow}
        locale={locale}
      />
      {state.capabilities.canModerate ? (
        <AdmissionQueue requests={admissionRequests} locale={locale} />
      ) : null}
    </div>
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
