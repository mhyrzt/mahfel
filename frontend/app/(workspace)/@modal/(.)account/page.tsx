import { cookies } from "next/headers";
import { getServerSession } from "next-auth";

import { AccountSettingsDialog } from "@/features/account/components/account-settings-dialog";
import { getCalendarPreference } from "@/features/account/calendar-preference";
import { isTheme, themeCookieName } from "@/features/appearance/theme";
import { getLocale } from "@/lib/i18n/server";
import { serverAuthOptions } from "@/lib/auth/options";

export default async function InterceptedAccountPage() {
  const [locale, cookieStore, session] = await Promise.all([
    getLocale(),
    cookies(),
    getServerSession(serverAuthOptions),
  ]);
  const themeCookie = cookieStore.get(themeCookieName)?.value;

  return (
    <AccountSettingsDialog
      calendar={getCalendarPreference(cookieStore.get("mahfel_calendar")?.value)}
      email={session?.user?.email ?? ""}
      image={session?.user?.image ?? null}
      locale={locale}
      name={session?.user?.name ?? ""}
      theme={isTheme(themeCookie) ? themeCookie : "system"}
    />
  );
}
