"use client";

import { calendarPreferenceCookie, type CalendarPreference } from "./calendar-preference";

export function persistCalendarPreference(calendar: CalendarPreference) {
  document.cookie = `${calendarPreferenceCookie}=${calendar}; Path=/; Max-Age=31536000; SameSite=Lax`;
}
