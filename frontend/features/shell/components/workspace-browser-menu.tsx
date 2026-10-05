"use client";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { useEffect, useState, type ReactNode } from "react";
import Link from "next/link";

import { Plus } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuSub,
  DropdownMenuSubTrigger,
  DropdownMenuSubContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
} from "@/components/ui/dropdown-menu";
import { CreateWorkspaceDialog } from "./create-workspace-dialog";
import { Skeleton } from "@/components/ui/skeleton";
import { roomCopy } from "@/features/rooms/room-copy";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

type Room = components["schemas"]["RoomResponse"];
type Session = components["schemas"]["SessionResponse"];

export function WorkspaceBrowserMenu({
  kind,
  locale,
  open,
  onOpenChange,
  children,
  mobile = false,
}: Readonly<{
  kind: "rooms" | "sessions";
  locale: Locale;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: ReactNode;
  mobile?: boolean;
}>) {
  const text = roomCopy(locale);
  const [rooms, setRooms] = useState<Room[] | null>(null);
  const [selectedRoomId, setSelectedRoomId] = useState<string | null>(null);
  const [sessions, setSessions] = useState<Session[] | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [manageableRoomId, setManageableRoomId] = useState<string | null>(null);
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

  useEffect(() => {
    if (!open || kind !== "sessions" || !selectedRoomId) {
      return undefined;
    }
    let active = true;
    void browserApiClient
      .get<{ can_manage_session: boolean }>(
        `/v1/rooms/${encodeURIComponent(selectedRoomId)}/membership`,
      )
      .then(({ data }) => {
        if (active) {
          setManageableRoomId(data.can_manage_session ? selectedRoomId : null);
        }
      })
      .catch(() => {
        if (active) {
          setManageableRoomId(null);
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
      <Alert variant="destructive" className="text-sm text-destructive">
        <AlertDescription>
          {kind === "rooms" ? text.unavailable : text.sessionUnavailable}
        </AlertDescription>
      </Alert>
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
      <ul className="space-y-1" data-testid="room-destinations">
        {rooms.map((room) => (
          <li key={room.id}>
            <DropdownMenuItem asChild className="min-h-10">
              <Link
                className="flex w-full min-w-0 rounded-md px-2 py-2 font-medium"
                href={`/room/${room.id}`}
                onClick={() => onOpenChange(false)}
              >
                {room.title}
              </Link>
            </DropdownMenuItem>
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
        <ul className="space-y-1" data-testid="session-destinations">
          {sessions.map((session) => (
            <li key={session.id}>
              <DropdownMenuItem asChild className="min-h-10">
                <Link
                  className="flex w-full min-w-0 rounded-md px-2 py-2 font-medium"
                  href={`/session/${session.id}`}
                  onClick={() => onOpenChange(false)}
                >
                  {session.title}
                </Link>
              </DropdownMenuItem>
            </li>
          ))}
        </ul>
      );
    }
    content = (
      <div className="space-y-5">
        <DropdownMenuSub>
          <DropdownMenuSubTrigger className="min-h-10">
            <span className="min-w-0 flex-1 truncate">
              {rooms.find((room) => room.id === selectedRoomId)?.title ?? text.roomSelection}
            </span>
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="max-h-80 w-64 overflow-y-auto">
            <DropdownMenuRadioGroup
              value={selectedRoomId ?? ""}
              onValueChange={(value) => {
                setSessions(null);
                setManageableRoomId(null);
                setSelectedRoomId(value);
              }}
            >
              {rooms.map((room) => (
                <DropdownMenuRadioItem
                  key={room.id}
                  value={room.id}
                  onSelect={(event) => event.preventDefault()}
                >
                  {room.title}
                </DropdownMenuRadioItem>
              ))}
            </DropdownMenuRadioGroup>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
        {sessionContent}
      </div>
    );
  }
  const menuSide = locale === "fa" ? "left" : "right";
  const canCreate =
    kind === "rooms"
      ? rooms !== null && !error
      : manageableRoomId !== null && manageableRoomId === selectedRoomId && !error;
  return (
    <>
      <DropdownMenu
        modal={false}
        dir={locale === "fa" ? "rtl" : "ltr"}
        open={open}
        onOpenChange={(nextOpen) => {
          setManageableRoomId(null);
          onOpenChange(nextOpen);
        }}
      >
        <DropdownMenuTrigger asChild>{children}</DropdownMenuTrigger>
        <DropdownMenuContent
          align="start"
          side={mobile ? "top" : menuSide}
          sideOffset={10}
          className="w-72 max-w-[calc(100vw-2rem)] rounded-xl p-2"
          data-testid={`${kind}-browser`}
        >
          <DropdownMenuLabel>{title}</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <div className="max-h-[min(60dvh,24rem)] overflow-y-auto p-1">{content}</div>
          {hasMoreRooms && !error ? (
            <DropdownMenuItem
              disabled={loadingMore}
              onSelect={(event) => {
                event.preventDefault();
                void loadMoreRooms();
              }}
            >
              {text.loadMoreRooms}
            </DropdownMenuItem>
          ) : null}
          {canCreate ? (
            <>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="min-h-10 text-primary"
                onSelect={() => setCreateOpen(true)}
              >
                <Plus aria-hidden="true" />
                {kind === "rooms" ? text.create : text.createSession}
              </DropdownMenuItem>
            </>
          ) : null}
        </DropdownMenuContent>
      </DropdownMenu>
      {createOpen ? (
        <CreateWorkspaceDialog
          kind={kind === "rooms" ? "room" : "session"}
          locale={locale}
          open={createOpen}
          onOpenChange={setCreateOpen}
          initialRoomId={selectedRoomId ?? undefined}
          initialRooms={rooms ?? undefined}
        />
      ) : null}
    </>
  );
}
