"use client";

import { useState } from "react";

import { Button } from "@/components/ui/button";
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
      <Button onClick={() => setOpen(true)}>{roomCopy(locale).newSession}</Button>
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
