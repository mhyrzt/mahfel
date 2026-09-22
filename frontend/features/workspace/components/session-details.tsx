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

function Details({ session }: Readonly<{ session: ScheduledSession }>) {
  return (
    <div className="space-y-5">
      <div className="grid gap-3 text-sm">
        <p className="flex items-center gap-2 text-muted-foreground">
          <CalendarClock className="size-4" aria-hidden="true" />
          {session.startsAt} · {session.duration}
        </p>
        <p className="flex items-center gap-2 text-muted-foreground">
          <UsersRound className="size-4" aria-hidden="true" />
          {session.participantCount} expected participants
        </p>
      </div>
      <Button className="w-full">View session</Button>
    </div>
  );
}

export function SessionDetails({ session }: Readonly<{ session: ScheduledSession }>) {
  return (
    <>
      <Dialog>
        <DialogTrigger asChild>
          <button
            className="hidden w-full rounded-xl text-left outline-none focus-visible:ring-2 focus-visible:ring-ring md:block"
            type="button"
          >
            <SessionAgendaRow session={session} />
          </button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{session.title}</DialogTitle>
            <DialogDescription>{session.roomName}</DialogDescription>
          </DialogHeader>
          <Details session={session} />
        </DialogContent>
      </Dialog>
      <Sheet>
        <SheetTrigger asChild>
          <button
            className="w-full rounded-xl text-left outline-none focus-visible:ring-2 focus-visible:ring-ring md:hidden"
            type="button"
          >
            <SessionAgendaRow session={session} />
          </button>
        </SheetTrigger>
        <SheetContent side="bottom">
          <SheetHeader>
            <SheetTitle>{session.title}</SheetTitle>
            <SheetDescription>{session.roomName}</SheetDescription>
          </SheetHeader>
          <div className="mt-5">
            <Details session={session} />
          </div>
        </SheetContent>
      </Sheet>
    </>
  );
}

function SessionAgendaRow({ session }: Readonly<{ session: ScheduledSession }>) {
  return (
    <div className="flex gap-3 rounded-xl border border-transparent p-3 transition-colors hover:border-border hover:bg-muted/70">
      <span className="w-11 shrink-0 pt-0.5 text-xs font-medium tabular-nums text-muted-foreground">
        {session.startsAt}
      </span>
      <div className="min-w-0">
        <p className="truncate text-sm font-medium">{session.title}</p>
        <p className="truncate text-xs text-muted-foreground">
          {session.roomName} · {session.duration}
        </p>
      </div>
    </div>
  );
}
