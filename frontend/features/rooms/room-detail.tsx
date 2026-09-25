import type { ReactNode } from "react";

import { RoomChat } from "@/features/chat/components/room-chat";
import { ContextFileLibrary } from "@/features/files/components/context-file-library";
import { InviteManager } from "@/features/invites/components/invite-manager";
import { CreateSessionButton } from "@/features/sessions/components/create-session-button";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";
import { RoomLobby } from "./room-lobby";
import { RoomAdmissionSettings } from "./room-admission-settings";
import { RoomSessions } from "./room-sessions";

export { RoomSessions } from "./room-sessions";

type Room = components["schemas"]["RoomResponse"];
type Session = components["schemas"]["SessionResponse"];

export function RoomDetail({
  room,
  sessions,
  role,
  canManageSession,
  actorId,
  locale,
}: Readonly<{
  room: Room;
  sessions: Session[];
  role: "manager" | "member";
  canManageSession: boolean;
  actorId?: string;
  locale: Locale;
}>) {
  return (
    <RoomDetailView
      room={room}
      sessions={sessions}
      locale={locale}
      canManageSession={canManageSession}
      chat={
        <RoomChat
          roomId={room.id}
          locale={locale}
          actorId={actorId}
          canModerate={role === "manager"}
        />
      }
      files={
        <ContextFileLibrary
          context={{ kind: "room", id: room.id }}
          locale={locale}
          canDelete={role === "manager"}
          actorId={actorId}
        />
      }
      invites={
        role === "manager" ? (
          <InviteManager target="room" targetId={room.id} locale={locale} />
        ) : null
      }
      lobby={role === "manager" ? <RoomLobby roomId={room.id} locale={locale} /> : null}
      admissionSettings={
        role === "manager" ? (
          <RoomAdmissionSettings roomId={room.id} initialPolicy={room.admission} locale={locale} />
        ) : null
      }
    />
  );
}

export function RoomDetailView({
  room,
  sessions,
  locale,
  canManageSession,
  chat,
  files,
  invites,
  lobby,
  admissionSettings,
}: Readonly<{
  room: Room;
  sessions: Session[];
  locale: Locale;
  canManageSession: boolean;
  chat?: ReactNode;
  files?: ReactNode;
  invites?: ReactNode;
  lobby?: ReactNode;
  admissionSettings?: ReactNode;
}>) {
  return (
    <div className="mx-auto max-w-7xl space-y-8 pb-16" data-testid="room-detail">
      <RoomHeader room={room} locale={locale} canManageSession={canManageSession} />
      <div className="grid items-start gap-10 xl:grid-cols-[minmax(0,1fr)_19rem]">
        <div className="min-w-0 space-y-12">
          <RoomSessions sessions={sessions} locale={locale} />
          {chat}
        </div>
        <aside
          className="min-w-0 space-y-8 xl:border-s xl:ps-8"
          aria-label={roomCopy(locale).roomTools}
        >
          {files}
          {invites}
          {lobby}
          {admissionSettings}
        </aside>
      </div>
    </div>
  );
}

export function RoomHeader({
  room,
  locale,
  canManageSession,
}: Readonly<{ room: Room; locale: Locale; canManageSession: boolean }>) {
  const text = roomCopy(locale);
  return (
    <header className="border-b pb-7 pt-2">
      <div className="flex flex-wrap items-end justify-between gap-6">
        <div className="min-w-0 space-y-3">
          <p className="text-xs font-semibold tracking-wide text-muted-foreground">{text.room}</p>
          <h1 className="max-w-3xl text-balance text-3xl font-semibold leading-tight tracking-tight sm:text-4xl">
            {room.title}
          </h1>
          <p className="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-muted-foreground">
            <span>{room.visibility === "invite_only" ? text.inviteOnly : text.public}</span>
            <span aria-hidden="true">·</span>
            <span>{room.admission === "lobby" ? text.lobby : text.openAdmission}</span>
          </p>
        </div>
        {canManageSession ? <CreateSessionButton room={room} locale={locale} /> : null}
      </div>
    </header>
  );
}
