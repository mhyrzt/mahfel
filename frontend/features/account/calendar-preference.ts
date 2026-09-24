export const calendarPreferenceCookie = "mahfel_calendar";

export const calendarPreferences = ["gregorian", "persian"] as const;

export type CalendarPreference = (typeof calendarPreferences)[number];

export function getCalendarPreference(value: string | undefined): CalendarPreference {
  return calendarPreferences.find((calendar) => calendar === value) ?? "gregorian";
}
