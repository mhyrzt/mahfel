"use client";

import { useState } from "react";
import { Plus } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { roomCopy } from "@/features/rooms/room-copy";
import { CreateWorkspaceDialog } from "@/features/shell/components/create-workspace-dialog";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

export function CreateSessionButton({
  room,
  locale,
}: Readonly<{ room: components["schemas"]["RoomResponse"]; locale: Locale }>) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              aria-label={roomCopy(locale).newSession}
              data-testid="new-session-trigger"
              onClick={() => setOpen(true)}
              size="icon"
            >
              <Plus aria-hidden="true" className="size-5" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>{roomCopy(locale).newSession}</TooltipContent>
        </Tooltip>
      </TooltipProvider>
      <CreateWorkspaceDialog
        initialRoomId={room.id}
        initialRooms={[room]}
        initialCanManage
        kind="session"
        locale={locale}
        onOpenChange={setOpen}
        open={open}
      />
    </>
  );
}
