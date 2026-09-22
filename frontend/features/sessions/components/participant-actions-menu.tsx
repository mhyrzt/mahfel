import { Ban, MicOff, MoreHorizontal, UserRoundX } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

import type { Participant, ParticipantAction } from "./session-types";

interface ParticipantActionsMenuProps {
  actions: ParticipantAction[];
  participant: Participant;
}

export function ParticipantActionsMenu({
  actions,
  participant,
}: Readonly<ParticipantActionsMenuProps>) {
  if (actions.length === 0) {
    return null;
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          aria-label={`Actions for ${participant.name}`}
          className="text-white hover:bg-white/15 hover:text-white"
          size="icon"
          variant="ghost"
        >
          <MoreHorizontal className="size-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-44">
        <DropdownMenuLabel>{participant.name}</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {actions.includes("mute") ? (
          <DropdownMenuItem>
            <MicOff className="size-4" />
            Mute participant
          </DropdownMenuItem>
        ) : null}
        {actions.includes("remove") ? (
          <DropdownMenuItem className="text-destructive focus:text-destructive">
            <UserRoundX className="size-4" />
            Remove from session
          </DropdownMenuItem>
        ) : null}
        {actions.includes("block") ? (
          <DropdownMenuItem className="text-destructive focus:text-destructive">
            <Ban className="size-4" />
            Block from room
          </DropdownMenuItem>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
