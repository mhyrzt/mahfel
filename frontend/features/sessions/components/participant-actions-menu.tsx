import { Ban, MicOff, MoreHorizontal, Settings, UserRoundX } from "lucide-react";

import { Button } from "@/components/ui/button";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { formatMessage, messagesFor } from "@/lib/i18n/messages";
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
  locale?: Locale;
  tone?: "light" | "dark";
  compact?: boolean;
  icon?: "more" | "settings";
  onAction?: (action: ParticipantAction, participant: Participant) => void;
}

export function ParticipantActionsMenu({
  actions,
  participant,
  locale = defaultLocale,
  tone = "light",
  compact = false,
  icon = "more",
  onAction,
}: Readonly<ParticipantActionsMenuProps>) {
  if (actions.length === 0) {
    return null;
  }
  const text = messagesFor(locale).sessions;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          aria-label={formatMessage(text.participantActionsLabel, { name: participant.name })}
          className={`${compact ? "size-8" : ""} ${
            icon === "settings" ? "rounded-full bg-background/90 shadow-sm ring-1 ring-border" : ""
          } ${
            tone === "dark"
              ? "text-white hover:bg-white/15 hover:text-white"
              : "text-foreground hover:bg-muted hover:text-foreground"
          }`}
          size="icon"
          variant="ghost"
        >
          {icon === "settings" ? (
            <Settings className="size-4" />
          ) : (
            <MoreHorizontal className="size-4" />
          )}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-44">
        <DropdownMenuLabel>{participant.name}</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {actions.includes("mute") ? (
          <DropdownMenuItem onSelect={() => onAction?.("mute", participant)}>
            <MicOff className="size-4" />
            {text.muteParticipant}
          </DropdownMenuItem>
        ) : null}
        {actions.includes("remove") ? (
          <DropdownMenuItem
            className="text-destructive focus:text-destructive"
            onSelect={() => onAction?.("remove", participant)}
          >
            <UserRoundX className="size-4" />
            {text.removeParticipant}
          </DropdownMenuItem>
        ) : null}
        {actions.includes("block") ? (
          <DropdownMenuItem
            className="text-destructive focus:text-destructive"
            onSelect={() => onAction?.("block", participant)}
          >
            <Ban className="size-4" />
            {text.blockParticipant}
          </DropdownMenuItem>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
