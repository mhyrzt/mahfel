import { cookies } from "next/headers";
import { getServerSession } from "next-auth";

import { AccountSettingsPanel } from "@/features/account/components/account-settings-panel";
import { getCalendarPreference } from "@/features/account/calendar-preference";
import { getProfile } from "@/features/account/profile";
import { isTheme, themeCookieName } from "@/features/appearance/theme";
import { getLocale } from "@/lib/i18n/server";
import { serverAuthOptions } from "@/lib/auth/options";

export default async function AccountPage() {
  const [locale, cookieStore, session, profile] = await Promise.all([
    getLocale(),
    cookies(),
    getServerSession(serverAuthOptions),
    getProfile(),
  ]);
  const themeCookie = cookieStore.get(themeCookieName)?.value;

  return (
    <AccountSettingsPanel
      calendar={getCalendarPreference(cookieStore.get("mahfel_calendar")?.value)}
      email={profile?.email ?? session?.user?.email ?? ""}
      image={profile?.avatar_url ?? session?.user?.image ?? null}
      locale={locale}
      name={profile?.display_name ?? session?.user?.name ?? ""}
      theme={isTheme(themeCookie) ? themeCookie : "system"}
    />
  );
}
