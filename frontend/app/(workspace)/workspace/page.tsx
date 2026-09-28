import { redirect } from "next/navigation";

import { WorkspaceEntry } from "@/features/shell/components/workspace-entry";
import { createServerApiClient, ServerApiAuthenticationError } from "@/lib/api/server";
import { WorkspaceHome } from "@/features/workspace/components/workspace-home";
import { loadWorkspaceOverview } from "@/features/workspace/workspace-data";
import { getLocale } from "@/lib/i18n/server";
import { cookies } from "next/headers";
import {
  calendarPreferenceCookie,
  getCalendarPreference,
} from "@/features/account/calendar-preference";
import { getProfile } from "@/features/account/profile";
import { getServerSessionUser } from "@/lib/auth/server";

export default async function WorkspacePage() {
  const [locale, cookieStore, profile, user] = await Promise.all([
    getLocale(),
    cookies(),
    getProfile(),
    getServerSessionUser(),
  ]);
  let overview: Awaited<ReturnType<typeof loadWorkspaceOverview>>;
  try {
    const api = await createServerApiClient();
    overview = await loadWorkspaceOverview(api);
  } catch (error) {
    if (error instanceof ServerApiAuthenticationError) {
      redirect("/login?callbackUrl=%2Fworkspace");
    }
    throw error;
  }
  if (overview.rooms.length === 0) {
    return <WorkspaceEntry locale={locale} />;
  }
  return (
    <WorkspaceHome
      accountName={profile?.display_name ?? user?.name}
      calendar={getCalendarPreference(cookieStore.get(calendarPreferenceCookie)?.value)}
      locale={locale}
      referenceDate={new Date().toISOString()}
      rooms={overview.rooms}
      scheduleUnavailable={overview.scheduleUnavailable}
      sessions={overview.sessions}
    />
  );
}
