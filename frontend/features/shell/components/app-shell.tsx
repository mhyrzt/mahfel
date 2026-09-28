import type { ReactNode } from "react";
import Image from "next/image";
import Link from "next/link";

import type { CalendarPreference } from "@/features/account/calendar-preference";
import type { Theme } from "@/features/appearance/theme";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { AccountQuickSettings } from "./account-quick-settings";
import { SidebarNavigation } from "./sidebar-navigation";
import { MobileNavigation } from "./mobile-navigation";
import { SidebarThemeToggle } from "./sidebar-theme-toggle";

export function AppShell({
  children,
  modal,
  locale = defaultLocale,
  calendar = "gregorian",
  theme = "system",
  accountName,
  accountEmail,
  accountImage,
}: Readonly<{
  children: ReactNode;
  modal?: ReactNode;
  locale?: Locale;
  calendar?: CalendarPreference;
  theme?: Theme;
  accountName?: string | null;
  accountEmail?: string;
  accountImage?: string | null;
}>) {
  return (
    <div className="min-h-screen bg-muted/40">
      <aside
        className="fixed inset-y-0 start-0 z-20 hidden w-16 flex-col items-center border-e bg-sidebar md:flex"
        dir={locale === "fa" ? "rtl" : "ltr"}
      >
        <Link
          aria-label="Mahfel home"
          className="mt-2 flex size-12 items-center justify-center rounded-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          href="/workspace"
        >
          <Image alt="" height={40} priority src="/mahfel-mark.svg" width={40} />
        </Link>
        <div className="shrink-0">
          <SidebarNavigation locale={locale} />
        </div>
        <div
          id="session-sidebar-actions"
          className="flex min-h-0 w-full flex-1 flex-col items-center justify-center gap-2 overflow-y-auto"
        />
        <div className="mt-auto flex w-full shrink-0 flex-col items-center gap-2 pb-2">
          <SidebarThemeToggle initialTheme={theme} locale={locale} />
          <AccountQuickSettings
            accountImage={accountImage}
            email={accountEmail}
            accountName={accountName}
            calendar={calendar}
            locale={locale}
            theme={theme}
          />
        </div>
      </aside>
      <div className="md:ps-16">
        <main className="w-full p-4 pb-[calc(4rem+env(safe-area-inset-bottom))] md:p-8">
          {children}
        </main>
      </div>
      <MobileNavigation locale={locale} accountName={accountName} accountImage={accountImage} />
      {modal}
    </div>
  );
}
