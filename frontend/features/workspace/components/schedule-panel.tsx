"use client";

import { useState, useSyncExternalStore } from "react";
import { CalendarDays } from "lucide-react";

import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import type { CalendarPreference } from "@/features/account/calendar-preference";
import type { Locale } from "@/lib/i18n/config";

import { MonthCalendar } from "./month-calendar";
import { SessionDetails } from "./session-details";
import type { ScheduledSession } from "./workspace-types";

interface SchedulePanelProps {
  sessions: ScheduledSession[];
  calendar: CalendarPreference;
  locale: Locale;
  referenceDate: string;
  scheduleUnavailable: boolean;
}

function dateKey(value: string, timeZone: string): string {
  const parts = new Intl.DateTimeFormat("en-CA", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    timeZone,
  }).formatToParts(new Date(value));
  const part = (type: string) => parts.find((item) => item.type === type)?.value;
  return `${part("year")}-${part("month")}-${part("day")}`;
}

const subscribeToTimeZone = () => () => {};
const getBrowserTimeZone = () => Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
const getServerTimeZone = () => "UTC";

export function SchedulePanel({
  sessions,
  calendar,
  locale,
  referenceDate,
  scheduleUnavailable,
}: Readonly<SchedulePanelProps>) {
  const timeZone = useSyncExternalStore(subscribeToTimeZone, getBrowserTimeZone, getServerTimeZone);
  const today = dateKey(referenceDate, timeZone);
  const [chosenDate, setChosenDate] = useState<string | null>(null);
  const [chosenMonth, setChosenMonth] = useState<string | null>(null);
  const selectedDate = chosenDate ?? today;
  const monthDate = chosenMonth ?? today;

  const activeDates = sessions.map((session) => dateKey(session.scheduledAt, timeZone));
  const selectedSessions = sessions.filter(
    (session) => dateKey(session.scheduledAt, timeZone) === selectedDate,
  );
  let label = locale === "fa" ? "امروز" : "Today";
  if (selectedDate !== today) {
    label = new Intl.DateTimeFormat(
      `${locale === "fa" ? "fa-IR" : "en-US"}-u-ca-${calendar === "persian" ? "persian" : "gregory"}`,
      {
        month: "long",
        day: "numeric",
        timeZone: "UTC",
      },
    ).format(new Date(`${selectedDate}T12:00:00Z`));
  }

  return (
    <Card className="shadow-none">
      <CardContent className="p-5">
        <MonthCalendar
          activeDates={activeDates}
          calendar={calendar}
          locale={locale}
          monthDate={monthDate}
          onMonthChange={(date) => {
            setChosenMonth(date);
            setChosenDate(date);
          }}
          onSelectDate={setChosenDate}
          selectedDate={selectedDate}
          today={today}
        />
        <Separator className="my-5" />
        <section aria-labelledby="workspace-schedule" className="space-y-2">
          <div className="flex items-center justify-between">
            <CardTitle className="text-base" id="workspace-schedule">
              {label}
            </CardTitle>
            <CalendarDays className="size-4 text-muted-foreground" aria-hidden="true" />
          </div>
          {scheduleUnavailable && (
            <p className="py-6 text-sm text-muted-foreground">
              {locale === "fa"
                ? "برنامهٔ جلسات فعلاً در دسترس نیست. اتاق‌ها را می‌توانید باز کنید."
                : "The schedule is temporarily unavailable. You can still open your rooms."}
            </p>
          )}
          {!scheduleUnavailable && selectedSessions.length === 0 && (
            <p className="py-6 text-sm text-muted-foreground">
              {locale === "fa"
                ? "جلسه‌ای برای این روز برنامه‌ریزی نشده است."
                : "No sessions are scheduled for this day."}
            </p>
          )}
          {!scheduleUnavailable && selectedSessions.length > 0 && (
            <div className="space-y-1">
              {selectedSessions.map((session) => (
                <SessionDetails
                  key={session.id}
                  locale={locale}
                  session={session}
                  timeZone={timeZone}
                />
              ))}
            </div>
          )}
        </section>
      </CardContent>
    </Card>
  );
}
