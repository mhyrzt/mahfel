"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { CalendarDays, CalendarPlus, DoorOpen, Plus, SquarePlus } from "lucide-react";

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

const itemClassName =
  "flex min-w-0 flex-col items-center justify-center gap-1 rounded-md px-2 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring";

export function MobileNavigation({ locale }: Readonly<{ locale: Locale }>) {
  const pathname = usePathname() ?? "";
  const text = messagesFor(locale).shell;
  const isRoomsActive = pathname.startsWith("/rooms");
  const isSessionsActive = pathname.startsWith("/sessions");

  return (
    <nav
      aria-label={text.mobileNavigation}
      className="fixed inset-x-0 bottom-0 z-30 border-t bg-background/95 pb-[env(safe-area-inset-bottom)] backdrop-blur lg:hidden"
    >
      <div className="mx-auto grid h-16 max-w-lg grid-cols-3 px-3">
        <Link
          aria-current={isRoomsActive ? "page" : undefined}
          className={`${itemClassName} ${isRoomsActive ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"}`}
          href="/rooms"
        >
          <DoorOpen aria-hidden="true" className="size-5" strokeWidth={1.8} />
          <span>{text.rooms}</span>
        </Link>
        <Link
          aria-current={isSessionsActive ? "page" : undefined}
          className={`${itemClassName} ${isSessionsActive ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"}`}
          href="/sessions"
        >
          <CalendarDays aria-hidden="true" className="size-5" strokeWidth={1.8} />
          <span>{text.sessions}</span>
        </Link>
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
                <Link
                  className="flex min-h-14 items-center gap-3 rounded-lg border p-4 text-sm font-medium transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  href="/rooms/new"
                >
                  <SquarePlus aria-hidden="true" className="size-5 text-muted-foreground" />
                  {text.createRoom}
                </Link>
              </SheetClose>
              <SheetClose asChild>
                <Link
                  className="flex min-h-14 items-center gap-3 rounded-lg border p-4 text-sm font-medium transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  href="/sessions/new"
                >
                  <CalendarPlus aria-hidden="true" className="size-5 text-muted-foreground" />
                  {text.scheduleSession}
                </Link>
              </SheetClose>
            </div>
          </SheetContent>
        </Sheet>
      </div>
    </nav>
  );
}
