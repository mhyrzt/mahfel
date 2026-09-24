import { CalendarDays } from "lucide-react";

import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

import { MonthCalendar } from "./month-calendar";
import { SessionDetails } from "./session-details";
import type { ScheduledSession } from "./workspace-types";

interface SchedulePanelProps {
  sessions: ScheduledSession[];
}

export function SchedulePanel({ sessions }: Readonly<SchedulePanelProps>) {
  return (
    <Card className="shadow-none">
      <CardContent className="p-5">
        <MonthCalendar activeDays={[4, 9, 14, 22, 24, 27]} selectedDay={22} />
        <Separator className="my-5" />
        <section aria-labelledby="today-schedule" className="space-y-2">
          <div className="flex items-center justify-between">
            <CardTitle className="text-base" id="today-schedule">
              Today
            </CardTitle>
            <CalendarDays className="size-4 text-muted-foreground" aria-hidden="true" />
          </div>
          {sessions.length === 0 ? (
            <p className="py-6 text-sm text-muted-foreground">No sessions are scheduled today.</p>
          ) : (
            <div className="space-y-1">
              {sessions.map((session) => (
                <SessionDetails key={session.id} session={session} />
              ))}
            </div>
          )}
        </section>
      </CardContent>
    </Card>
  );
}
