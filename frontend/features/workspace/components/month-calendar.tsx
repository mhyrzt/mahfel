import { ChevronLeft, ChevronRight } from "lucide-react";

import { Button } from "@/components/ui/button";

interface MonthCalendarProps {
  activeDays: number[];
  selectedDay: number;
}

const weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const days = Array.from({ length: 30 }, (_, index) => index + 1);

export function MonthCalendar({ activeDays, selectedDay }: Readonly<MonthCalendarProps>) {
  return (
    <section aria-label="September 2026 calendar" className="space-y-4">
      <div className="flex items-center justify-between">
        <Button aria-label="Previous month" size="icon" variant="ghost">
          <ChevronLeft className="size-4" />
        </Button>
        <p className="text-sm font-semibold">September 2026</p>
        <Button aria-label="Next month" size="icon" variant="ghost">
          <ChevronRight className="size-4" />
        </Button>
      </div>
      <div className="grid grid-cols-7 gap-y-2 text-center text-xs">
        {weekdays.map((weekday) => (
          <span className="font-medium text-muted-foreground" key={weekday}>
            {weekday}
          </span>
        ))}
        <span />
        {days.map((day) => {
          const hasSessions = activeDays.includes(day);
          const selected = day === selectedDay;

          return (
            <button
              aria-current={selected ? "date" : undefined}
              className={`relative mx-auto grid size-8 place-items-center rounded-lg text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${
                selected ? "bg-primary font-semibold text-primary-foreground" : "hover:bg-muted"
              }`}
              key={day}
              type="button"
            >
              {day}
              {hasSessions && !selected ? (
                <span className="absolute bottom-0.5 size-1 rounded-full bg-primary" />
              ) : null}
            </button>
          );
        })}
      </div>
    </section>
  );
}
