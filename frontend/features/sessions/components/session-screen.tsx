import { AlertCircle, LoaderCircle, UserRoundCheck, UsersRound } from "lucide-react";

import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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
  SessionTiming,
} from "./session-types";
import { WaitingRoomPanel } from "./waiting-room-panel";

interface SessionScreenProps {
  title: string;
  state: SessionState;
  admissionRequests?: AdmissionRequest[];
  sessionTiming?: SessionTiming | null;
  locale?: Locale;
  onRetryJoin?: () => void;
  onParticipantAction?: (action: ParticipantAction, participant: Participant) => void;
  actionError?: string | null;
  canAdmit?: boolean;
  onAdmit?: (request: AdmissionRequest) => void;
  onReject?: (request: AdmissionRequest) => void;
  onLeave?: () => void;
}

export function SessionScreen({
  title,
  state,
  admissionRequests = [],
  sessionTiming,
  locale = defaultLocale,
  onRetryJoin,
  onParticipantAction,
  actionError,
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
  return (
    <div className="space-y-6">
      {actionError ? (
        <p
          className="rounded-lg border border-destructive/40 p-3 text-sm text-destructive"
          role="alert"
        >
          {actionError}
        </p>
      ) : null}
      <div
        className="grid gap-6 xl:grid-cols-[20rem_minmax(0,1fr)]"
        dir={locale === "fa" ? "rtl" : "ltr"}
      >
        <Tabs
          defaultValue="participants"
          className="order-2 min-w-0 xl:order-1 xl:flex xl:h-[calc(100svh-9rem)] xl:flex-col"
          dir={locale === "fa" ? "rtl" : "ltr"}
        >
          <TabsList
            className={`grid h-auto w-full ${showAdmission ? "grid-cols-2" : "grid-cols-1"}`}
          >
            <TabsTrigger
              aria-label={text.participants}
              className="px-2"
              title={text.participants}
              value="participants"
            >
              <UsersRound className="size-4" aria-hidden="true" />
            </TabsTrigger>
            {showAdmission ? (
              <TabsTrigger
                aria-label={text.admissionTab}
                className="px-2"
                title={text.admissionTab}
                value="admission"
              >
                <UserRoundCheck className="size-4" aria-hidden="true" />
              </TabsTrigger>
            ) : null}
          </TabsList>
          <TabsContent className="h-[32rem] xl:min-h-0 xl:flex-1 xl:h-auto" value="participants">
            <ParticipantRoster
              participants={state.participants}
              summary={
                sessionTiming ? <SessionSummary timing={sessionTiming} locale={locale} /> : null
              }
              locale={locale}
            />
          </TabsContent>
          {showAdmission ? (
            <TabsContent className="h-[32rem] xl:min-h-0 xl:flex-1 xl:h-auto" value="admission">
              <AdmissionQueue
                requests={admissionRequests}
                locale={locale}
                onAdmit={onAdmit}
                onReject={onReject}
              />
            </TabsContent>
          ) : null}
        </Tabs>
        <div className="order-1 min-w-0 xl:order-2">
          <SessionStage
            participantActions={
              isLive && state.capabilities.canModerate ? ["mute", "remove", "block"] : []
            }
            participants={state.participants}
            status={isLive ? "live" : "ended"}
            title={title}
            toolbarCapabilities={isLive ? state.capabilities : undefined}
            locale={locale}
            onParticipantAction={onParticipantAction}
            onLeave={onLeave}
          />
        </div>
      </div>
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
