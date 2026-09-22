import { MicOff, VideoOff } from "lucide-react";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";

import { ParticipantActionsMenu } from "./participant-actions-menu";
import type { Participant, ParticipantAction } from "./session-types";

interface ParticipantTileProps {
  actions?: ParticipantAction[];
  participant: Participant;
}

export function ParticipantTile({ actions = [], participant }: Readonly<ParticipantTileProps>) {
  return (
    <article
      className={`relative flex aspect-video min-h-36 flex-col justify-between overflow-hidden rounded-lg border bg-slate-950 p-4 text-white ${
        participant.speaking ? "ring-2 ring-primary ring-offset-2" : ""
      }`}
    >
      <div className="flex items-start justify-between">
        <span className="rounded-full bg-white/15 px-2 py-1 text-xs font-medium capitalize">
          {participant.role}
        </span>
        <ParticipantActionsMenu actions={actions} participant={participant} />
      </div>
      <div className="flex items-end justify-between gap-3">
        <div className="flex items-center gap-2">
          <Avatar className="size-9 border border-white/20">
            <AvatarFallback className="bg-white/15 text-xs font-semibold text-white">
              {participant.initials}
            </AvatarFallback>
          </Avatar>
          <span className="truncate text-sm font-medium">{participant.name}</span>
        </div>
        <div className="flex items-center gap-1 text-white/80">
          {participant.muted ? <MicOff className="size-4" aria-label="Muted" /> : null}
          {!participant.videoEnabled ? (
            <VideoOff className="size-4" aria-label="Video off" />
          ) : null}
        </div>
      </div>
    </article>
  );
}
