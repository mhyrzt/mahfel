import { cookies } from "next/headers";
import { getServerSession } from "next-auth";

import { AccountSettingsPanel } from "@/features/account/components/account-settings-panel";
import { getCalendarPreference } from "@/features/account/calendar-preference";
import { isTheme, themeCookieName } from "@/features/appearance/theme";
import { getLocale } from "@/lib/i18n/server";
import { serverAuthOptions } from "@/lib/auth/options";

export default async function AccountPage() {
  const [locale, cookieStore, session] = await Promise.all([
    getLocale(),
    cookies(),
    getServerSession(serverAuthOptions),
  ]);
  const themeCookie = cookieStore.get(themeCookieName)?.value;

  return (
    <AccountSettingsPanel
      calendar={getCalendarPreference(cookieStore.get("mahfel_calendar")?.value)}
      email={session?.user?.email ?? ""}
      image={session?.user?.image ?? null}
      locale={locale}
      name={session?.user?.name ?? ""}
      theme={isTheme(themeCookie) ? themeCookie : "system"}
    />
  );
}
