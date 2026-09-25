import { notFound, redirect } from "next/navigation";
import axios from "axios";

import { RoomDetail } from "@/features/rooms/room-detail";
import { createServerApiClient, ServerApiAuthenticationError } from "@/lib/api/server";
import type { components } from "@/lib/api/generated/schema";
import { getLocale } from "@/lib/i18n/server";
import { getServerSessionUser } from "@/lib/auth/server";

type Room = components["schemas"]["RoomResponse"];
type Session = components["schemas"]["SessionResponse"];

export default async function RoomPage({
  params,
}: Readonly<{ params: Promise<{ roomId: string }> }>) {
  const [{ roomId }, locale, user] = await Promise.all([
    params,
    getLocale(),
    getServerSessionUser(),
  ]);
  let room: Room;
  let sessions: Session[];
  let role: "manager" | "member";
  let canManageSession: boolean;
  try {
    const api = await createServerApiClient();
    const [roomResponse, sessionsResponse, membershipResponse] = await Promise.all([
      api.get<Room>(`/v1/rooms/${encodeURIComponent(roomId)}`),
      api.get<Session[]>(`/v1/rooms/${encodeURIComponent(roomId)}/sessions`),
      api.get<{ role: "manager" | "member"; can_manage_session: boolean }>(
        `/v1/rooms/${encodeURIComponent(roomId)}/membership`,
      ),
    ]);
    room = roomResponse.data;
    sessions = sessionsResponse.data;
    role = membershipResponse.data.role;
    canManageSession = membershipResponse.data.can_manage_session;
  } catch (error) {
    if (error instanceof ServerApiAuthenticationError) {
      redirect(`/login?callbackUrl=${encodeURIComponent(`/room/${roomId}`)}`);
    }
    if (axios.isAxiosError(error) && error.response?.status === 404) {
      notFound();
    }
    throw error;
  }
  return (
    <RoomDetail
      room={room}
      sessions={sessions}
      role={role}
      canManageSession={canManageSession}
      actorId={user?.userId}
      locale={locale}
    />
  );
}
