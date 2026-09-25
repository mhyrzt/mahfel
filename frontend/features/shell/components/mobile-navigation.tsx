"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { CalendarClock, CalendarPlus, PanelsTopLeft, Plus, SquarePlus } from "lucide-react";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import { CreateWorkspaceDialog } from "./create-workspace-dialog";
import { WorkspaceBrowserDialog } from "./workspace-browser-dialog";

const itemClassName =
  "flex min-w-0 flex-col items-center justify-center gap-1 rounded-md px-2 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring";

export function MobileNavigation({
  locale,
  accountName,
  accountImage,
}: Readonly<{ locale: Locale; accountName?: string | null; accountImage?: string | null }>) {
  const pathname = usePathname() ?? "";
  const [browseKind, setBrowseKind] = useState<"rooms" | "sessions" | null>(null);
  const [createKind, setCreateKind] = useState<"room" | "session" | null>(null);
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
      <div className="mx-auto grid h-16 max-w-lg grid-cols-4 px-3">
        <Button
          aria-current={isRoomsActive ? "page" : undefined}
          className={`${itemClassName} h-full w-full ${isRoomsActive ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"}`}
          onClick={() => setBrowseKind("rooms")}
          variant="ghost"
        >
          <PanelsTopLeft aria-hidden="true" className="size-5" strokeWidth={1.8} />
          <span>{text.rooms}</span>
        </Button>
        <Button
          aria-current={isSessionsActive ? "page" : undefined}
          className={`${itemClassName} h-full w-full ${isSessionsActive ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"}`}
          onClick={() => setBrowseKind("sessions")}
          variant="ghost"
        >
          <CalendarClock aria-hidden="true" className="size-5" strokeWidth={1.8} />
          <span>{text.sessions}</span>
        </Button>
        <Sheet>
          <SheetTrigger asChild>
            <Button
              aria-label={text.create}
              className={`${itemClassName} h-full w-full text-muted-foreground hover:text-accent-foreground`}
              variant="ghost"
            >
              <Plus aria-hidden="true" className="size-5" strokeWidth={1.8} />
              <span>{text.create}</span>
            </Button>
          </SheetTrigger>
          <SheetContent
            className="rounded-t-2xl pb-[max(env(safe-area-inset-bottom),1rem)]"
            side="bottom"
          >
            <SheetHeader className="text-start">
              <SheetTitle>{text.createMenuTitle}</SheetTitle>
              <SheetDescription>{text.createMenuDescription}</SheetDescription>
            </SheetHeader>
            <div className="mt-6 grid gap-3">
              <SheetClose asChild>
                <Button
                  className="flex min-h-14 items-center gap-3 rounded-lg border p-4 text-sm font-medium transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  onClick={() => setCreateKind("room")}
                  variant="ghost"
                >
                  <SquarePlus aria-hidden="true" className="size-5 text-muted-foreground" />
                  {text.createRoom}
                </Button>
              </SheetClose>
              <SheetClose asChild>
                <Button
                  className="flex min-h-14 items-center gap-3 rounded-lg border p-4 text-sm font-medium transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  onClick={() => setCreateKind("session")}
                  variant="ghost"
                >
                  <CalendarPlus aria-hidden="true" className="size-5 text-muted-foreground" />
                  {text.scheduleSession}
                </Button>
              </SheetClose>
            </div>
          </SheetContent>
        </Sheet>
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
      <WorkspaceBrowserDialog
        kind="rooms"
        locale={locale}
        onOpenChange={(open) => !open && setBrowseKind(null)}
        open={browseKind === "rooms"}
      />
      <WorkspaceBrowserDialog
        kind="sessions"
        locale={locale}
        onOpenChange={(open) => !open && setBrowseKind(null)}
        open={browseKind === "sessions"}
      />
      <CreateWorkspaceDialog
        kind="room"
        locale={locale}
        onOpenChange={(open) => !open && setCreateKind(null)}
        open={createKind === "room"}
      />
      <CreateWorkspaceDialog
        kind="session"
        locale={locale}
        onOpenChange={(open) => !open && setCreateKind(null)}
        open={createKind === "session"}
      />
    </nav>
  );
}
