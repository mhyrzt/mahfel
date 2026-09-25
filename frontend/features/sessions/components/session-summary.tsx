"use client";

import { useEffect, useState } from "react";
import { CalendarDays, Clock3 } from "lucide-react";

import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import type { SessionTiming } from "./session-types";

interface SessionSummaryProps {
  timing: SessionTiming;
  locale?: Locale;
}

export function SessionSummary({ timing, locale = defaultLocale }: Readonly<SessionSummaryProps>) {
  const text = messagesFor(locale).sessions;
  const observedAt = timing.status === "available" ? Date.parse(timing.observedAt) : 0;
  const [currentTime, setCurrentTime] = useState(() =>
    Number.isFinite(observedAt) ? observedAt : 0,
  );

  useEffect(() => {
    if (timing.status !== "available" || timing.endedAt !== null) {
      return undefined;
    }
    const updateTime = () => setCurrentTime(Date.now());
    updateTime();
    const intervalId = window.setInterval(updateTime, 60_000);
    return () => window.clearInterval(intervalId);
  }, [timing]);

  if (timing.status !== "available" || !timing.startedAt) {
    return (
      <section aria-label={text.participationSummary} className="px-6 pb-4">
        <p className="text-sm text-muted-foreground">{text.sessionTimingUnavailable}</p>
      </section>
    );
  }

  const startedMilliseconds = Date.parse(timing.startedAt);
  const endedMilliseconds = timing.endedAt ? Date.parse(timing.endedAt) : currentTime;
  const language = locale === "fa" ? "fa-IR" : "en-US";
  const startedLabel = Number.isFinite(startedMilliseconds)
    ? new Intl.DateTimeFormat(language, { dateStyle: "medium", timeStyle: "short" }).format(
        startedMilliseconds,
      )
    : text.sessionTimingUnavailable;
  const elapsedSeconds = Math.max(0, Math.floor((endedMilliseconds - startedMilliseconds) / 1000));
  let durationLabel = text.sessionTimingUnavailable;
  if (Number.isFinite(startedMilliseconds) && Number.isFinite(endedMilliseconds)) {
    durationLabel =
      elapsedSeconds < 60 ? text.underMinute : formatDuration(elapsedSeconds, language);
  }

  return (
    <section aria-label={text.participationSummary} className="px-6 pb-4">
      <div className="grid gap-3">
        <div className="flex items-center gap-3">
          <CalendarDays className="size-4 text-muted-foreground" aria-hidden="true" />
          <div>
            <p className="text-xs text-muted-foreground">{text.started}</p>
            <time className="whitespace-nowrap text-sm font-medium" dateTime={timing.startedAt}>
              {startedLabel}
            </time>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <Clock3 className="size-4 text-muted-foreground" aria-hidden="true" />
          <div>
            <p className="text-xs text-muted-foreground">{text.duration}</p>
            <p className="text-sm font-medium">{durationLabel}</p>
          </div>
        </div>
      </div>
    </section>
  );
}

function formatDuration(seconds: number, locale: string) {
  const minutes = Math.floor(seconds / 60);
  const days = Math.floor(minutes / 1_440);
  const hours = Math.floor((minutes % 1_440) / 60);
  const remainingMinutes = minutes % 60;
  const units: string[] = [];

  if (days > 0) {
    units.push(
      new Intl.NumberFormat(locale, { style: "unit", unit: "day", unitDisplay: "long" }).format(
        days,
      ),
    );
  }
  if (hours > 0) {
    units.push(
      new Intl.NumberFormat(locale, {
        style: "unit",
        unit: "hour",
        unitDisplay: "long",
      }).format(hours),
    );
  }
  if (remainingMinutes > 0 || units.length === 0) {
    units.push(
      new Intl.NumberFormat(locale, {
        style: "unit",
        unit: "minute",
        unitDisplay: "long",
      }).format(remainingMinutes),
    );
  }

  return units.join(locale === "fa-IR" ? " و " : " ");
}
