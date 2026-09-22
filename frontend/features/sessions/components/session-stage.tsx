import { CalendarClock } from "lucide-react";

import { Card, CardContent } from "@/components/ui/card";

import { ParticipantTile } from "./participant-tile";
import { SessionStatusBadge } from "./session-status-badge";
import { SessionToolbar } from "./session-toolbar";
import type { Participant, ParticipantAction, SessionStatus } from "./session-types";

interface SessionStageProps {
  participantActions?: ParticipantAction[];
  title: string;
  status: SessionStatus;
  participants: Participant[];
  scheduledFor?: string;
}

export function SessionStage({
  title,
  status,
  participants,
  participantActions = [],
  scheduledFor,
}: Readonly<SessionStageProps>) {
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
              {scheduledFor ? `Scheduled for ${scheduledFor}` : "Scheduling details are not set."}
            </p>
          </div>
          <SessionStatusBadge status={status} />
        </CardContent>
      </Card>
    );
  }

  if (status === "ended") {
    return (
      <Card>
        <CardContent className="flex aspect-video flex-col items-center justify-center gap-3 text-center">
          <SessionStatusBadge status={status} />
          <div className="space-y-1">
            <p className="font-semibold">{title}</p>
            <p className="text-sm text-muted-foreground">This session has ended.</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <section className="space-y-4 pb-24 md:pb-0" aria-label={`${title} session stage`}>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">{title}</h1>
          <p className="text-sm text-muted-foreground">Live session</p>
        </div>
        <SessionStatusBadge status={status} />
      </div>
      <div className="grid gap-3 sm:grid-cols-2">
        {participants.map((participant) => (
          <ParticipantTile
            actions={participantActions}
            key={participant.id}
            participant={participant}
          />
        ))}
      </div>
      <div className="fixed inset-x-0 bottom-0 z-20 flex justify-center px-4 pb-[max(env(safe-area-inset-bottom),1rem)] md:static md:px-0 md:pb-0">
        <SessionToolbar />
      </div>
    </section>
  );
}
