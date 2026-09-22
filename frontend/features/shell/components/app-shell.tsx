import type { ReactNode } from "react";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { SidebarActions } from "./sidebar-actions";
import { SidebarNavigation } from "./sidebar-navigation";
import { MobileNavigation } from "./mobile-navigation";

export function AppShell({
  children,
  locale = defaultLocale,
}: Readonly<{ children: ReactNode; locale?: Locale }>) {
  return (
    <div className="min-h-screen bg-muted/40">
      <aside className="fixed inset-y-0 hidden w-16 flex-col bg-background lg:flex">
        <div className="h-16" aria-hidden="true" />
        <SidebarNavigation locale={locale} />
        <SidebarActions locale={locale} />
      </aside>
      <div className="lg:pl-16">
        <header className="sticky top-0 z-10 flex h-16 items-center justify-between border-b bg-background/95 px-4 backdrop-blur lg:px-8">
          <p className="text-lg font-semibold tracking-tight">Mahfel</p>
          <div className="flex items-center gap-2">
            <Avatar className="size-8">
              <AvatarFallback className="bg-primary text-xs font-semibold text-primary-foreground">
                R
              </AvatarFallback>
            </Avatar>
          </div>
        </header>
        <main className="mx-auto w-full max-w-7xl p-4 pb-[calc(4rem+env(safe-area-inset-bottom))] lg:p-8">
          {children}
        </main>
      </div>
      <MobileNavigation locale={locale} />
    </div>
  );
}
