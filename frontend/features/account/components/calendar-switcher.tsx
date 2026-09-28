"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";

import { SegmentedControl } from "@/components/ui/segmented-control";
import { messagesFor } from "@/lib/i18n/messages";
import type { Locale } from "@/lib/i18n/config";

import { calendarPreferences, type CalendarPreference } from "../calendar-preference";
import { persistCalendarPreference } from "../calendar-preference-client";

export function CalendarSwitcher({
  calendar,
  locale,
}: Readonly<{ calendar: CalendarPreference; locale: Locale }>) {
  const router = useRouter();
  const [selectedCalendar, setSelectedCalendar] = useState(calendar);
  const messages = messagesFor(locale).account;

  function changeCalendar(nextCalendar: CalendarPreference) {
    setSelectedCalendar(nextCalendar);
    persistCalendarPreference(nextCalendar);
    router.refresh();
  }

  return (
    <SegmentedControl
      label={messages.calendarType}
      onValueChange={changeCalendar}
      options={calendarPreferences.map((value) => ({ value, label: messages[value] }))}
      value={selectedCalendar}
    />
  );
}
