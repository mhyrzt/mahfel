import { ChevronLeft, ChevronRight } from "lucide-react";

import { Button } from "@/components/ui/button";
import type { CalendarPreference } from "@/features/account/calendar-preference";
import type { Locale } from "@/lib/i18n/config";

interface MonthCalendarProps {
  activeDates: string[];
  calendar: CalendarPreference;
  locale: Locale;
  monthDate: string;
  selectedDate: string;
  today: string;
  onMonthChange: (date: string) => void;
  onSelectDate: (date: string) => void;
}

const dateKey = (date: Date) => date.toISOString().slice(0, 10);
const persianPartsFormatter = new Intl.DateTimeFormat("en-US-u-ca-persian", {
  year: "numeric",
  month: "numeric",
  day: "numeric",
  timeZone: "UTC",
});

function persianParts(date: Date) {
  const parts = persianPartsFormatter.formatToParts(date);
  const value = (type: string) => Number(parts.find((part) => part.type === type)?.value);
  return { year: value("year"), month: value("month"), day: value("day") };
}

function startOfMonth(date: Date, calendar: CalendarPreference) {
  const first = new Date(date);
  first.setUTCDate(
    first.getUTCDate() -
      (calendar === "persian" ? persianParts(date).day - 1 : date.getUTCDate() - 1),
  );
  return first;
}

function nextMonth(first: Date, calendar: CalendarPreference) {
  if (calendar === "gregorian") {
    return new Date(Date.UTC(first.getUTCFullYear(), first.getUTCMonth() + 1, 1));
  }
  const current = persianParts(first);
  const next = new Date(first);
  do {
    next.setUTCDate(next.getUTCDate() + 1);
  } while (persianParts(next).year === current.year && persianParts(next).month === current.month);
  return next;
}

export function MonthCalendar({
  activeDates,
  calendar,
  locale,
  monthDate,
  selectedDate,
  today,
  onMonthChange,
  onSelectDate,
}: Readonly<MonthCalendarProps>) {
  const firstDay = startOfMonth(new Date(`${monthDate}T12:00:00Z`), calendar);
  const followingMonth = nextMonth(firstDay, calendar);
  const dayCount = Math.round((followingMonth.getTime() - firstDay.getTime()) / 86_400_000);
  const saturdayFirst = calendar === "persian" || locale === "fa";
  const offset = saturdayFirst ? (firstDay.getUTCDay() + 1) % 7 : (firstDay.getUTCDay() + 6) % 7;
  const calendarLocale = `${locale === "fa" ? "fa-IR" : "en-US"}-u-ca-${calendar === "persian" ? "persian" : "gregory"}`;
  const monthLabel = new Intl.DateTimeFormat(calendarLocale, {
    month: "long",
    year: "numeric",
    timeZone: "UTC",
  }).format(firstDay);
  const weekdayFormatter = new Intl.DateTimeFormat(calendarLocale, {
    weekday: "short",
    timeZone: "UTC",
  });
  const weekdays =
    locale === "fa"
      ? ["ش", "ی", "د", "س", "چ", "پ", "ج"]
      : Array.from({ length: 7 }, (_, index) =>
          weekdayFormatter.format(new Date(Date.UTC(2026, 8, (saturdayFirst ? 19 : 20) + index))),
        );
  const numberFormatter = new Intl.NumberFormat(locale === "fa" ? "fa-IR" : "en-US", {
    useGrouping: false,
  });
  const active = new Set(activeDates);

  const shiftMonth = (direction: -1 | 1) => {
    const target =
      direction === 1
        ? followingMonth
        : startOfMonth(new Date(firstDay.getTime() - 86_400_000), calendar);
    onMonthChange(dateKey(target));
  };

  return (
    <section aria-label={monthLabel} className="space-y-3">
      <div className="flex items-center justify-between">
        <Button
          aria-label={locale === "fa" ? "ماه قبل" : "Previous month"}
          className="size-8 rounded-lg"
          onClick={() => shiftMonth(-1)}
          size="icon"
          type="button"
          variant="ghost"
        >
          <ChevronLeft className="size-4 rtl:rotate-180" />
        </Button>
        <p className="text-sm font-medium tracking-tight">{monthLabel}</p>
        <Button
          aria-label={locale === "fa" ? "ماه بعد" : "Next month"}
          className="size-8 rounded-lg"
          onClick={() => shiftMonth(1)}
          size="icon"
          type="button"
          variant="ghost"
        >
          <ChevronRight className="size-4 rtl:rotate-180" />
        </Button>
      </div>
      <div className="grid grid-cols-7 gap-1 text-center">
        {weekdays.map((weekday, index) => (
          <span
            className="flex h-8 min-w-0 items-center justify-center text-[0.7rem] font-medium text-muted-foreground"
            key={index}
          >
            {weekday}
          </span>
        ))}
        {Array.from({ length: offset }, (_, index) => (
          <span key={`offset-${index}`} />
        ))}
        {Array.from({ length: dayCount }, (_, index) => {
          const current = new Date(firstDay);
          current.setUTCDate(current.getUTCDate() + index);
          const date = dateKey(current);
          const selected = date === selectedDate;
          const isToday = date === today;
          const day = calendar === "persian" ? persianParts(current).day : current.getUTCDate();
          let stateClass = "hover:bg-accent hover:text-accent-foreground";
          if (selected) {
            stateClass =
              "bg-primary font-semibold text-primary-foreground shadow-sm hover:bg-primary/90";
          } else if (isToday) {
            stateClass = "font-semibold text-primary ring-1 ring-primary/40 hover:bg-accent";
          }
          return (
            <Button
              variant="plain"
              aria-current={isToday ? "date" : undefined}
              aria-label={date}
              aria-pressed={selected}
              className={`relative mx-auto grid size-9 p-0 place-items-center rounded-lg text-sm tabular-nums transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 ${stateClass}`}
              key={date}
              onClick={() => onSelectDate(date)}
              type="button"
            >
              {numberFormatter.format(day)}
              {active.has(date) && !selected ? (
                <span className="absolute bottom-1 size-1 rounded-full bg-primary" />
              ) : null}
            </Button>
          );
        })}
      </div>
    </section>
  );
}
