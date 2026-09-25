"use client";

import { useEffect, useState, type ReactNode } from "react";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Skeleton } from "@/components/ui/skeleton";
import { roomCopy } from "@/features/rooms/room-copy";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

type Room = components["schemas"]["RoomResponse"];
type Session = components["schemas"]["SessionResponse"];

export function WorkspaceBrowserDialog({
  kind,
  locale,
  open,
  onOpenChange,
}: Readonly<{
  kind: "rooms" | "sessions";
  locale: Locale;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}>) {
  const text = roomCopy(locale);
  const [rooms, setRooms] = useState<Room[] | null>(null);
  const [selectedRoomId, setSelectedRoomId] = useState<string | null>(null);
  const [sessions, setSessions] = useState<Session[] | null>(null);
  const [error, setError] = useState(false);
  const [roomPage, setRoomPage] = useState(1);
  const [hasMoreRooms, setHasMoreRooms] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);

  useEffect(() => {
    if (!open) {
      return undefined;
    }
    let active = true;
    void browserApiClient
      .get<{ data: Room[] }>("/v1/rooms", { params: { page: 1, per_page: 100 } })
      .then(({ data }) => {
        if (active) {
          setRooms(data.data);
          setRoomPage(1);
          setHasMoreRooms(data.data.length === 100);
          setSelectedRoomId((current) =>
            data.data.some((room) => room.id === current) ? current : (data.data[0]?.id ?? null),
          );
          setError(false);
        }
      })
      .catch(() => {
        if (active) {
          setError(true);
        }
      });
    return () => {
      active = false;
    };
  }, [open]);

  useEffect(() => {
    if (!open || kind !== "sessions" || !selectedRoomId) {
      return undefined;
    }
    let active = true;
    void browserApiClient
      .get<Session[]>(`/v1/rooms/${encodeURIComponent(selectedRoomId)}/sessions`)
      .then(({ data }) => {
        if (active) {
          setSessions(data);
          setError(false);
        }
      })
      .catch(() => {
        if (active) {
          setError(true);
        }
      });
    return () => {
      active = false;
    };
  }, [kind, open, selectedRoomId]);

  async function loadMoreRooms() {
    if (loadingMore || !hasMoreRooms) {
      return;
    }
    setLoadingMore(true);
    try {
      const nextPage = roomPage + 1;
      const { data } = await browserApiClient.get<{ data: Room[] }>("/v1/rooms", {
        params: { page: nextPage, per_page: 100 },
      });
      setRooms((current) => [...(current ?? []), ...data.data]);
      setRoomPage(nextPage);
      setHasMoreRooms(data.data.length === 100);
      setError(false);
    } catch {
      setError(true);
    } finally {
      setLoadingMore(false);
    }
  }

  const title = kind === "rooms" ? text.rooms : text.sessions;
  let content: ReactNode;
  if (error) {
    content = (
      <p className="text-sm text-destructive" role="alert">
        {kind === "rooms" ? text.unavailable : text.sessionUnavailable}
      </p>
    );
  } else if (rooms === null) {
    content = (
      <div aria-busy="true" className="space-y-3">
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-14 w-full" />
        <Skeleton className="h-14 w-full" />
      </div>
    );
  } else if (rooms.length === 0) {
    content = <p className="text-sm text-muted-foreground">{text.empty}</p>;
  } else if (kind === "rooms") {
    content = (
      <ul className="divide-y border-y" data-testid="room-destinations">
        {rooms.map((room) => (
          <li key={room.id}>
            <Link
              className="block rounded-md py-4 font-medium hover:text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              href={`/room/${room.id}`}
              onClick={() => onOpenChange(false)}
            >
              {room.title}
            </Link>
          </li>
        ))}
      </ul>
    );
  } else {
    let sessionContent: ReactNode;
    if (sessions === null) {
      sessionContent = <Skeleton className="h-16 w-full" />;
    } else if (sessions.length === 0) {
      sessionContent = <p className="text-sm text-muted-foreground">{text.noSessions}</p>;
    } else {
      sessionContent = (
        <ul className="divide-y border-y" data-testid="session-destinations">
          {sessions.map((session) => (
            <li key={session.id}>
              <Link
                className="block rounded-md py-4 font-medium hover:text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                href={`/session/${session.id}`}
                onClick={() => onOpenChange(false)}
              >
                {session.title}
              </Link>
            </li>
          ))}
        </ul>
      );
    }
    content = (
      <div className="space-y-5">
        <div className="space-y-2">
          <label className="block text-sm font-medium" htmlFor="browser-room">
            {text.roomSelection}
          </label>
          <select
            className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm focus-visible:ring-2 focus-visible:ring-ring"
            id="browser-room"
            onChange={(event) => {
              setSessions(null);
              setSelectedRoomId(event.target.value);
            }}
            value={selectedRoomId ?? ""}
          >
            {rooms.map((room) => (
              <option key={room.id} value={room.id}>
                {room.title}
              </option>
            ))}
          </select>
        </div>
        {sessionContent}
      </div>
    );
  }
  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        className="max-h-[calc(100dvh-2rem)] overflow-y-auto"
        data-testid={`${kind}-browser`}
      >
        <DialogTitle>{title}</DialogTitle>
        <DialogDescription className="sr-only">{title}</DialogDescription>
        {content}
        {hasMoreRooms && !error ? (
          <Button disabled={loadingMore} onClick={() => void loadMoreRooms()} variant="outline">
            {text.loadMoreRooms}
          </Button>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}
