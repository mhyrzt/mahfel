"use client";

import { useEffect, useState, type ReactNode } from "react";

import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Skeleton } from "@/components/ui/skeleton";
import { CreateRoomForm } from "@/features/rooms/create-room-form";
import { roomCopy } from "@/features/rooms/room-copy";
import { CreateSessionForm } from "@/features/sessions/components/create-session-form";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

type Room = components["schemas"]["RoomResponse"];

export function CreateWorkspaceDialog({
  kind,
  locale,
  open,
  onOpenChange,
  initialRoomId,
  initialRooms,
  initialCanManage,
}: Readonly<{
  kind: "room" | "session";
  locale: Locale;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  initialRoomId?: string;
  initialRooms?: Room[];
  initialCanManage?: boolean;
}>) {
  const text = roomCopy(locale);
  const [rooms, setRooms] = useState<Room[] | null>(initialRooms ?? null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    if (!open || kind !== "session" || initialRooms) {
      return undefined;
    }
    let active = true;
    void browserApiClient
      .get<{ data: Room[] }>("/v1/rooms", { params: { page: 1, per_page: 100 } })
      .then(({ data }) => {
        if (active) {
          setRooms(data.data);
          setFailed(false);
        }
      })
      .catch(() => {
        if (active) {
          setFailed(true);
        }
      });
    return () => {
      active = false;
    };
  }, [initialRooms, kind, open]);

  let content: ReactNode;
  if (kind === "room") {
    content = <CreateRoomForm locale={locale} onCreated={() => onOpenChange(false)} />;
  } else if (failed) {
    content = (
      <p role="alert" className="text-sm text-destructive">
        {text.sessionUnavailable}
      </p>
    );
  } else if (rooms === null) {
    content = (
      <div aria-busy="true" className="space-y-5" data-testid="create-session-loading">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-10 w-32" />
      </div>
    );
  } else {
    content = (
      <CreateSessionForm
        initialRoomId={initialRoomId}
        initialCanManage={initialCanManage}
        key={initialRoomId ?? rooms[0]?.id}
        locale={locale}
        onCreated={() => onOpenChange(false)}
        rooms={rooms}
      />
    );
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent className="max-h-[calc(100dvh-2rem)] overflow-y-auto">
        <DialogTitle className="sr-only">
          {kind === "room" ? text.create : text.newSession}
        </DialogTitle>
        <DialogDescription className="sr-only">
          {kind === "room" ? text.title : text.roomSelection}
        </DialogDescription>
        {content}
      </DialogContent>
    </Dialog>
  );
}
