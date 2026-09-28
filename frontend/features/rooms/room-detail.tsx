import type { ReactNode } from "react";
import Link from "next/link";
import { ChevronRight, UsersRound } from "lucide-react";

import { InviteManager } from "@/features/invites/components/invite-manager";
import { RoomChat } from "@/features/chat/components/room-chat";
import { ConversationSidebar } from "@/features/chat/components/conversation-sidebar";
import { CreateSessionButton } from "@/features/sessions/components/create-session-button";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";
import { roomCopy } from "./room-copy";

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
      chat={
        <RoomChat
          roomId={room.id}
          locale={locale}
          actorId={actorId}
          canModerate={role === "manager"}
          compact
        />
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
  chat,
}: Readonly<{
  room: Room;
  sessions: Session[];
  locale: Locale;
  canManageSession: boolean;
  settings?: ReactNode;
  fileData?: Record<string, SessionFileFixture>;
  chat?: ReactNode;
}>) {
  return (
    <div className="mx-auto max-w-5xl space-y-6 pb-16" data-testid="room-detail">
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
      {chat ? <ConversationSidebar locale={locale} conversation={chat} /> : null}
    </div>
  );
}

export function RoomHeader({
  room,
  locale,
  canManageSession,
  settings,
}: Readonly<{ room: Room; locale: Locale; canManageSession: boolean; settings?: ReactNode }>) {
  const text = roomCopy(locale);
  return (
    <header className="rounded-2xl border bg-card px-5 py-5 sm:px-7 sm:py-6">
      <nav
        aria-label={locale === "fa" ? "مسیر صفحه" : "Breadcrumb"}
        className="mb-5 flex items-center gap-1.5 text-xs text-muted-foreground"
      >
        <Link className="hover:text-foreground" href="/workspace">
          {locale === "fa" ? "خانه" : "Home"}
        </Link>
        <ChevronRight aria-hidden="true" className="size-3.5 rtl:rotate-180" />
        <span aria-current="page" className="truncate text-foreground">
          {room.title}
        </span>
      </nav>
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div className="min-w-0 space-y-2">
          <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
            <UsersRound aria-hidden="true" className="size-4" />
            {room.visibility === "public" ? text.public : text.inviteOnly}
          </div>
          <h1 className="min-w-0 break-words text-2xl font-semibold tracking-tight sm:text-4xl">
            {room.title}
          </h1>
        </div>
        <div className="flex shrink-0 flex-wrap items-center gap-2">
          {settings}
          {canManageSession ? <CreateSessionButton room={room} locale={locale} /> : null}
        </div>
      </div>
    </header>
  );
}
