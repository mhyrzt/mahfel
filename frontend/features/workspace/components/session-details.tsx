import Link from "next/link";
import { CalendarClock, UsersRound } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";

import type { ScheduledSession } from "./workspace-types";
import type { Locale } from "@/lib/i18n/config";

interface SessionDetailsProps {
  session: ScheduledSession;
  locale: Locale;
  timeZone: string;
}

function sessionTime(session: ScheduledSession, locale: Locale, timeZone: string) {
  return new Intl.DateTimeFormat(locale === "fa" ? "fa-IR" : "en-US", {
    hour: "numeric",
    minute: "2-digit",
    timeZone,
  }).format(new Date(session.scheduledAt));
}

function Details({ session, locale, timeZone }: Readonly<SessionDetailsProps>) {
  return (
    <div className="space-y-5">
      <div className="grid gap-3 text-sm">
        <p className="flex items-center gap-2 text-muted-foreground">
          <CalendarClock className="size-4" aria-hidden="true" />
          {sessionTime(session, locale, timeZone)}
          {session.durationMinutes
            ? ` · ${session.durationMinutes} ${locale === "fa" ? "دقیقه" : "min"}`
            : null}
        </p>
        {session.participantCount !== undefined ? (
          <p className="flex items-center gap-2 text-muted-foreground">
            <UsersRound className="size-4" aria-hidden="true" />
            {session.participantCount}{" "}
            {locale === "fa" ? "شرکت‌کنندهٔ مورد انتظار" : "expected participants"}
          </p>
        ) : null}
      </div>
      <Button asChild className="w-full">
        <Link href={`/session/${session.id}`}>
          {locale === "fa" ? "مشاهدهٔ جلسه" : "View session"}
        </Link>
      </Button>
    </div>
  );
}

export function SessionDetails({ session, locale, timeZone }: Readonly<SessionDetailsProps>) {
  return (
    <>
      <Dialog>
        <DialogTrigger asChild>
          <Button
            variant="plain"
            className="hidden h-auto w-full whitespace-normal p-0 rounded-xl text-start outline-none focus-visible:ring-2 focus-visible:ring-ring md:block"
            type="button"
          >
            <SessionAgendaRow locale={locale} session={session} timeZone={timeZone} />
          </Button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{session.title}</DialogTitle>
            <DialogDescription>{session.roomName}</DialogDescription>
          </DialogHeader>
          <Details locale={locale} session={session} timeZone={timeZone} />
        </DialogContent>
      </Dialog>
      <Sheet>
        <SheetTrigger asChild>
          <Button
            variant="plain"
            className="block h-auto w-full whitespace-normal p-0 rounded-xl text-start outline-none focus-visible:ring-2 focus-visible:ring-ring md:hidden"
            type="button"
          >
            <SessionAgendaRow locale={locale} session={session} timeZone={timeZone} />
          </Button>
        </SheetTrigger>
        <SheetContent side="bottom">
          <SheetHeader>
            <SheetTitle>{session.title}</SheetTitle>
            <SheetDescription>{session.roomName}</SheetDescription>
          </SheetHeader>
          <div className="mt-5">
            <Details locale={locale} session={session} timeZone={timeZone} />
          </div>
        </SheetContent>
      </Sheet>
    </>
  );
}

function SessionAgendaRow({ session, locale, timeZone }: Readonly<SessionDetailsProps>) {
  return (
    <div className="flex gap-3 rounded-xl border border-transparent p-3 transition-colors hover:border-border hover:bg-muted/70">
      <span className="w-11 shrink-0 pt-0.5 text-xs font-medium tabular-nums text-muted-foreground">
        {sessionTime(session, locale, timeZone)}
      </span>
      <div className="min-w-0">
        <p className="truncate text-sm font-medium">{session.title}</p>
        <p className="truncate text-xs text-muted-foreground">
          {session.roomName}
          {session.durationMinutes
            ? ` · ${session.durationMinutes} ${locale === "fa" ? "دقیقه" : "min"}`
            : null}
        </p>
      </div>
    </div>
  );
}
