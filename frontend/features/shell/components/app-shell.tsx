import type { ReactNode } from "react";
import Image from "next/image";
import Link from "next/link";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import { SidebarActions } from "./sidebar-actions";
import { SidebarNavigation } from "./sidebar-navigation";
import { MobileNavigation } from "./mobile-navigation";

export function AppShell({
  children,
  modal,
  locale = defaultLocale,
  accountName,
  accountImage,
}: Readonly<{
  children: ReactNode;
  modal?: ReactNode;
  locale?: Locale;
  accountName?: string | null;
  accountImage?: string | null;
}>) {
  const accountInitial = accountName?.trim().slice(0, 1).toLocaleUpperCase(locale) || "?";
  const accountLabel = messagesFor(locale).account.openSettings;
  return (
    <div className="min-h-screen bg-muted/40">
      <aside className="fixed inset-y-0 hidden w-16 flex-col bg-background lg:flex">
        <div className="h-16" aria-hidden="true" />
        <SidebarNavigation locale={locale} />
        <SidebarActions locale={locale} />
      </aside>
      <div className="lg:pl-16">
        <header className="sticky top-0 z-10 flex h-16 items-center justify-between border-b bg-background/95 px-0 backdrop-blur lg:-ml-16 lg:w-[calc(100%+4rem)]">
          <Link
            aria-label="Mahfel home"
            className="flex items-center gap-2 rounded-md text-lg font-semibold tracking-tight focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            href="/rooms"
          >
            <Image
              alt=""
              className="h-12 w-20 object-contain"
              height={45}
              priority
              src="/mahfel-mark.svg"
              width={80}
            />
          </Link>
          <Link
            aria-label={accountLabel}
            className="mr-4 rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring lg:mr-8"
            href="/account"
          >
            <Avatar className="size-8">
              <AvatarImage alt="" src={accountImage ?? undefined} />
              <AvatarFallback className="bg-primary text-xs font-semibold text-primary-foreground">
                {accountInitial}
              </AvatarFallback>
            </Avatar>
          </Link>
        </header>
        <main className="mx-auto w-full max-w-7xl p-4 pb-[calc(4rem+env(safe-area-inset-bottom))] lg:p-8">
          {children}
        </main>
      </div>
      <MobileNavigation locale={locale} />
      {modal}
    </div>
  );
}
