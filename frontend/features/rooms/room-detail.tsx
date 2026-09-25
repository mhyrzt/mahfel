import type { ReactNode } from "react";

import { InviteManager } from "@/features/invites/components/invite-manager";
import { CreateSessionButton } from "@/features/sessions/components/create-session-button";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { RoomLobby } from "./room-lobby";
import { RoomAdmissionSettings } from "./room-admission-settings";
import { RoomMembers } from "./room-members";
import { RoomSessions, type SessionFileFixture } from "./room-sessions";
import { RoomSettingsDialog } from "./room-settings-dialog";

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
      settings={
        role === "manager" ? (
          <RoomSettingsDialog
            locale={locale}
            invites={<InviteManager target="room" targetId={room.id} locale={locale} />}
            requests={
              <>
                <RoomAdmissionSettings
                  roomId={room.id}
                  initialPolicy={room.admission}
                  locale={locale}
                />
                <RoomLobby roomId={room.id} locale={locale} />
              </>
            }
            members={<RoomMembers roomId={room.id} actorId={actorId} locale={locale} />}
          />
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
  settings,
  fileData,
}: Readonly<{
  room: Room;
  sessions: Session[];
  locale: Locale;
  canManageSession: boolean;
  settings?: ReactNode;
  fileData?: Record<string, SessionFileFixture>;
}>) {
  return (
    <div className="mx-auto max-w-5xl space-y-8 pb-16" data-testid="room-detail">
      <RoomHeader
        room={room}
        locale={locale}
        canManageSession={canManageSession}
        settings={settings}
      />
      <RoomSessions
        sessions={sessions}
        locale={locale}
        fileData={fileData}
        canManageSession={canManageSession}
      />
    </div>
  );
}

export function RoomHeader({
  room,
  locale,
  canManageSession,
  settings,
}: Readonly<{ room: Room; locale: Locale; canManageSession: boolean; settings?: ReactNode }>) {
  return (
    <header className="flex flex-wrap items-end justify-between gap-4 border-b pb-5 pt-2">
      <h1 className="min-w-0 break-words text-2xl font-semibold tracking-tight sm:text-3xl">
        {room.title}
      </h1>
      <div className="flex shrink-0 flex-wrap items-center gap-2">
        {settings}
        {canManageSession ? <CreateSessionButton room={room} locale={locale} /> : null}
      </div>
    </header>
  );
}
