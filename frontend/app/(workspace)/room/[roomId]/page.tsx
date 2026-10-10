import { notFound, redirect } from "next/navigation";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { roomCopy } from "@/features/rooms/room-copy";

import { RoomDetail } from "@/features/rooms/room-detail";
import {
  createServerApiClient,
  ServerApiAuthenticationError,
  ServerApiRequestError,
} from "@/lib/api/server";
import type { components } from "@/lib/api/generated/schema";
import { getLocale } from "@/lib/i18n/server";
import { getServerSessionUser } from "@/lib/auth/server";

type Room = components["schemas"]["RoomResponse"];
type Session = components["schemas"]["SessionResponse"];
type SessionPage = components["schemas"]["Paginated_SessionResponse"];

export default async function RoomPage({
  params,
  searchParams,
}: Readonly<{
  params: Promise<{ roomId: string }>;
  searchParams: Promise<{ page?: string | string[] }>;
}>) {
  const [{ roomId }, locale, user] = await Promise.all([
    params,
    getLocale(),
    getServerSessionUser(),
  ]);
  const query = await searchParams;
  const rawPage = Array.isArray(query.page) ? query.page[0] : query.page;
  const numericPage = Number(rawPage);
  const page = Number.isSafeInteger(numericPage) && numericPage > 0 ? numericPage : 1;
  const text = roomCopy(locale);
  let room: Room;
  let sessions: Session[];
  let role: "manager" | "member";
  let canManageSession: boolean;
  try {
    const api = await createServerApiClient();
    const [roomResponse, sessionsResponse, membershipResponse] = await Promise.all([
      api.get<Room>(`/v1/rooms/${encodeURIComponent(roomId)}`),
      api.get<SessionPage>(`/v1/rooms/${encodeURIComponent(roomId)}/sessions`, {
        params: { page, per_page: 20 },
      }),
      api.get<{ role: "manager" | "member"; can_manage_session: boolean }>(
        `/v1/rooms/${encodeURIComponent(roomId)}/membership`,
      ),
    ]);
    room = roomResponse.data;
    sessions = sessionsResponse.data.data;
    role = membershipResponse.data.role;
    canManageSession = membershipResponse.data.can_manage_session;
  } catch (error) {
    if (error instanceof ServerApiAuthenticationError) {
      redirect(`/login?callbackUrl=${encodeURIComponent(`/room/${roomId}`)}`);
    }
    if (error instanceof ServerApiRequestError && error.status === 404) {
      notFound();
    }
    throw error;
  }
  return (
    <RoomDetail
      room={room}
      sessions={sessions}
      sessionNavigation={
        page > 1 || sessions.length === 20 ? (
          <nav aria-label={text.sessions} className="flex flex-wrap justify-between gap-3">
            {page > 1 ? (
              <Button asChild variant="outline">
                <Link href={`/room/${encodeURIComponent(roomId)}?page=${page - 1}`}>
                  {text.newerSessions}
                </Link>
              </Button>
            ) : (
              <span />
            )}
            {sessions.length === 20 ? (
              <Button asChild variant="outline">
                <Link href={`/room/${encodeURIComponent(roomId)}?page=${page + 1}`}>
                  {text.olderSessions}
                </Link>
              </Button>
            ) : null}
          </nav>
        ) : undefined
      }
      role={role}
      canManageSession={canManageSession}
      actorId={user?.userId}
      locale={locale}
    />
  );
}
