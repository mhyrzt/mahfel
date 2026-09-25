import { redirect } from "next/navigation";

import { WorkspaceEntry } from "@/features/shell/components/workspace-entry";
import { createServerApiClient, ServerApiAuthenticationError } from "@/lib/api/server";
import type { components } from "@/lib/api/generated/schema";
import { getLocale } from "@/lib/i18n/server";

type Room = components["schemas"]["RoomResponse"];

export default async function WorkspacePage() {
  const locale = await getLocale();
  let rooms: Room[];
  try {
    const api = await createServerApiClient();
    const { data } = await api.get<{ data: Room[] }>("/v1/rooms", {
      params: { page: 1, per_page: 1 },
    });
    rooms = data.data;
  } catch (error) {
    if (error instanceof ServerApiAuthenticationError) {
      redirect("/login?callbackUrl=%2Fworkspace");
    }
    throw error;
  }
  if (rooms[0]) {
    redirect(`/room/${rooms[0].id}`);
  }
  return <WorkspaceEntry locale={locale} />;
}
