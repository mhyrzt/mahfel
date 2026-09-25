import type { ReactNode } from "react";
import Link from "next/link";

import { RoomChat } from "@/features/chat/components/room-chat";
import { ContextFileLibrary } from "@/features/files/components/context-file-library";
import { InviteManager } from "@/features/invites/components/invite-manager";
import { CreateSessionButton } from "@/features/sessions/components/create-session-button";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";
import { RoomLobby } from "./room-lobby";
import { RoomAdmissionSettings } from "./room-admission-settings";

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
    <div className="space-y-10" data-testid="room-detail">
      <RoomHeader room={room} locale={locale} canManageSession={canManageSession} />
      <RoomSessions sessions={sessions} locale={locale} />
      {chat}
      {files}
      {invites}
      {lobby}
      {admissionSettings}
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
    <header className="border-b pb-8">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div className="space-y-2">
          <h1 className="text-3xl font-semibold tracking-tight">{room.title}</h1>
          <p className="text-sm text-muted-foreground">
            {room.visibility === "invite_only" ? text.inviteOnly : text.public}
            {" · "}
            {room.admission === "lobby" ? text.lobby : text.openAdmission}
          </p>
        </div>
        {canManageSession ? <CreateSessionButton room={room} locale={locale} /> : null}
      </div>
    </header>
  );
}

export function RoomSessions({
  sessions,
  locale,
}: Readonly<{ sessions: Session[]; locale: Locale }>) {
  const text = roomCopy(locale);
  return (
    <section aria-labelledby="room-sessions" data-testid="room-sessions" className="space-y-4">
      <h2 className="text-lg font-semibold" id="room-sessions">
        {text.sessions}
      </h2>
      {sessions.length === 0 ? (
        <p className="text-sm text-muted-foreground">{text.noSessions}</p>
      ) : (
        <ul className="divide-y border-y">
          {sessions.map((session) => (
            <li key={session.id}>
              <Link
                className="flex min-h-16 items-center justify-between gap-4 py-3 outline-none hover:text-primary focus-visible:ring-2 focus-visible:ring-ring"
                href={`/session/${session.id}`}
              >
                <span className="min-w-0 truncate font-medium">{session.title}</span>
                <span className="shrink-0 text-xs capitalize text-muted-foreground">
                  {session.status}
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
