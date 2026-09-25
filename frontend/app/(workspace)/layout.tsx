import type { ReactNode } from "react";
import { cookies } from "next/headers";

import { AppShell } from "@/features/shell/components/app-shell";
import {
  calendarPreferenceCookie,
  getCalendarPreference,
} from "@/features/account/calendar-preference";
import { isTheme, themeCookieName } from "@/features/appearance/theme";
import { getProfile } from "@/features/account/profile";
import { getServerSessionUser } from "@/lib/auth/server";
import { getLocale } from "@/lib/i18n/server";

export default async function WorkspaceLayout({
  children,
  modal,
}: Readonly<{ children: ReactNode; modal: ReactNode }>) {
  const [locale, user, profile, cookieStore] = await Promise.all([
    getLocale(),
    getServerSessionUser(),
    getProfile(),
    cookies(),
  ]);
  const themeCookie = cookieStore.get(themeCookieName)?.value;
  return (
    <AppShell
      accountImage={profile?.avatar_url ?? user?.image}
      accountEmail={user?.email ?? ""}
      accountName={profile?.display_name ?? user?.name}
      calendar={getCalendarPreference(cookieStore.get(calendarPreferenceCookie)?.value)}
      locale={locale}
      modal={modal}
      theme={isTheme(themeCookie) ? themeCookie : "system"}
    >
      {children}
    </AppShell>
  );
}
