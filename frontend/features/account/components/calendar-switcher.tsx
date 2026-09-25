"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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

  function changeCalendar(value: string) {
    const nextCalendar = calendarPreferences.find((item) => item === value);
    if (!nextCalendar) {
      return;
    }
    setSelectedCalendar(nextCalendar);
    persistCalendarPreference(nextCalendar);
    router.refresh();
  }

  return (
    <Select onValueChange={changeCalendar} value={selectedCalendar}>
      <SelectTrigger aria-label={messages.calendarType}>
        <SelectValue />
      </SelectTrigger>
      <SelectContent dir={locale === "fa" ? "rtl" : "ltr"}>
        {calendarPreferences.map((availableCalendar) => (
          <SelectItem key={availableCalendar} value={availableCalendar}>
            {messages[availableCalendar]}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
