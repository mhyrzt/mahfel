import "server-only";

import type { AxiosInstance } from "axios";

import type { components } from "@/lib/api/generated/schema";
import { ServerApiAuthenticationError } from "@/lib/api/server";

import type { Room, ScheduledSession } from "./components/workspace-types";

type ApiRoom = components["schemas"]["RoomResponse"];
type ApiSession = components["schemas"]["SessionResponse"];
type RoomPage = components["schemas"]["Paginated_RoomResponse"];

const overviewRoomLimit = 12;

export async function loadWorkspaceOverview(api: AxiosInstance): Promise<{
  rooms: Room[];
  sessions: ScheduledSession[];
  scheduleUnavailable: boolean;
}> {
  const response = await api.get<RoomPage>("/v1/rooms", {
    params: { page: 1, per_page: overviewRoomLimit },
  });
  const apiRooms: ApiRoom[] = response.data.data;
  const sessionResults = await Promise.allSettled(
    apiRooms.map(async (room) => {
      const result = await api.get<ApiSession[]>(
        `/v1/rooms/${encodeURIComponent(room.id)}/sessions`,
      );
      return result.data;
    }),
  );
  for (const result of sessionResults) {
    if (result.status === "rejected" && result.reason instanceof ServerApiAuthenticationError) {
      throw result.reason;
    }
  }
  const sessionsByRoom = sessionResults.map((result) =>
    result.status === "fulfilled" ? result.value : [],
  );
  const scheduleUnavailable = sessionResults.some((result) => result.status === "rejected");

  const rooms = apiRooms.map((room, index): Room => {
    const sessions = sessionsByRoom[index];
    const live = sessions.some((session) => session.status === "live");
    const scheduled = sessions.some((session) => session.status === "scheduled");
    let status: Room["status"] = "idle";
    if (live) {
      status = "live";
    } else if (scheduled) {
      status = "scheduled";
    }
    return {
      id: room.id,
      name: room.title,
      status,
    };
  });
  const sessions = apiRooms
    .flatMap((room, index) =>
      sessionsByRoom[index].flatMap((session): ScheduledSession[] =>
        session.status === "scheduled" && session.scheduled_start_at
          ? [
              {
                id: session.id,
                title: session.title,
                roomName: room.title,
                scheduledAt: session.scheduled_start_at,
                durationMinutes: session.scheduled_duration_minutes ?? null,
              },
            ]
          : [],
      ),
    )
    .sort((left, right) => left.scheduledAt.localeCompare(right.scheduledAt));

  return { rooms, sessions, scheduleUnavailable };
}
