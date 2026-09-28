"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { CalendarClock, PanelsTopLeft } from "lucide-react";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import { WorkspaceBrowserMenu } from "./workspace-browser-menu";

const itemClassName =
  "flex min-w-0 flex-col items-center justify-center gap-1 rounded-md px-2 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring";

export function MobileNavigation({
  locale,
  accountName,
  accountImage,
}: Readonly<{ locale: Locale; accountName?: string | null; accountImage?: string | null }>) {
  const pathname = usePathname() ?? "";
  const [browseKind, setBrowseKind] = useState<"rooms" | "sessions" | null>(null);
  const text = messagesFor(locale).shell;
  const isRoomsActive = pathname.startsWith("/room/");
  const isSessionsActive = pathname.startsWith("/session/");
  const isAccountActive = pathname.startsWith("/account");
  const accountLabel = messagesFor(locale).account.openSettings;
  const accountInitial = accountName?.trim().slice(0, 1).toLocaleUpperCase(locale) || "?";

  return (
    <nav
      aria-label={text.mobileNavigation}
      className="fixed inset-x-0 bottom-0 z-30 border-t bg-background/95 pb-[env(safe-area-inset-bottom)] backdrop-blur md:hidden"
    >
      <div className="mx-auto grid h-16 max-w-lg grid-cols-3 px-3">
        <WorkspaceBrowserMenu
          kind="rooms"
          locale={locale}
          mobile
          open={browseKind === "rooms"}
          onOpenChange={(open) => setBrowseKind(open ? "rooms" : null)}
        >
          <Button
            aria-current={isRoomsActive ? "page" : undefined}
            className={`${itemClassName} h-full w-full ${isRoomsActive ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"}`}
            variant="ghost"
          >
            <PanelsTopLeft aria-hidden="true" className="size-5" strokeWidth={1.8} />
            <span>{text.rooms}</span>
          </Button>
        </WorkspaceBrowserMenu>
        <WorkspaceBrowserMenu
          kind="sessions"
          locale={locale}
          mobile
          open={browseKind === "sessions"}
          onOpenChange={(open) => setBrowseKind(open ? "sessions" : null)}
        >
          <Button
            aria-current={isSessionsActive ? "page" : undefined}
            className={`${itemClassName} h-full w-full ${isSessionsActive ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"}`}
            variant="ghost"
          >
            <CalendarClock aria-hidden="true" className="size-5" strokeWidth={1.8} />
            <span>{text.sessions}</span>
          </Button>
        </WorkspaceBrowserMenu>
        <Button
          asChild
          aria-label={accountLabel}
          className={`${itemClassName} h-full w-full ${isAccountActive ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"}`}
          variant="ghost"
        >
          <Link aria-current={isAccountActive ? "page" : undefined} href="/account">
            <Avatar className="size-5">
              <AvatarImage alt="" src={accountImage ?? undefined} />
              <AvatarFallback className="bg-primary text-[0.55rem] font-semibold text-primary-foreground">
                {accountInitial}
              </AvatarFallback>
            </Avatar>
            <span className="whitespace-normal">{text.account}</span>
          </Link>
        </Button>
      </div>
    </nav>
  );
}
