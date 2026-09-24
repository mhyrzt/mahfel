"use client";

import { useState } from "react";
import { CalendarDays, Languages, SunMoon } from "lucide-react";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { DevicePreferences } from "@/features/account/components/device-preferences";
import {
  calendarPreferences,
  calendarPreferenceCookie,
  type CalendarPreference,
} from "@/features/account/calendar-preference";
import { ThemeSwitcher } from "@/features/appearance/components/theme-switcher";
import type { Theme } from "@/features/appearance/theme";
import { LocaleSwitcher } from "@/features/i18n/components/locale-switcher";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

interface AccountSettingsPanelProps {
  calendar: CalendarPreference;
  email: string;
  image: string | null;
  locale: Locale;
  name: string;
  theme: Theme;
}

export function AccountSettingsPanel({
  calendar,
  email,
  image,
  locale,
  name,
  theme,
}: Readonly<AccountSettingsPanelProps>) {
  const [selectedCalendar, setSelectedCalendar] = useState(calendar);
  const messages = messagesFor(locale).account;
  const initials = name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toLocaleUpperCase(locale))
    .join("");

  function changeCalendar(value: string) {
    const nextCalendar = calendarPreferences.find((item) => item === value);
    if (!nextCalendar) {
      return;
    }
    setSelectedCalendar(nextCalendar);
    document.cookie = `${calendarPreferenceCookie}=${nextCalendar}; Path=/; Max-Age=31536000; SameSite=Lax`;
  }

  return (
    <div className="space-y-6">
      <header>
        <h1 className="text-xl font-semibold">{messages.title}</h1>
        <p className="mt-1 text-sm text-muted-foreground">{messages.description}</p>
      </header>
      <section className="grid gap-6 rounded-xl border p-5 sm:grid-cols-[minmax(9rem,0.7fr)_minmax(0,1.3fr)]">
        <div className="flex flex-col items-center gap-3 sm:items-start">
          <h2 className="text-sm font-medium">{messages.picture}</h2>
          <Avatar className="size-28 text-3xl">
            <AvatarImage alt={messages.picture} src={image ?? undefined} />
            <AvatarFallback className="bg-primary text-primary-foreground">
              {initials || "?"}
            </AvatarFallback>
          </Avatar>
        </div>
        <div className="space-y-5">
          <h2 className="font-medium">{messages.profile}</h2>
          <ProfileField label={messages.name} value={name || messages.notSet} />
          <ProfileField label={messages.email} value={email || messages.notSet} />
        </div>
      </section>
      <section className="grid gap-4 rounded-xl border p-5 sm:grid-cols-3">
        <SettingField icon={SunMoon} label={messagesFor(locale).appearance.label}>
          <ThemeSwitcher locale={locale} theme={theme} />
        </SettingField>
        <SettingField icon={CalendarDays} label={messages.calendarType}>
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
        </SettingField>
        <SettingField icon={Languages} label={messages.language}>
          <LocaleSwitcher locale={locale} />
        </SettingField>
      </section>
      <DevicePreferences locale={locale} />
    </div>
  );
}

function ProfileField({ label, value }: Readonly<{ label: string; value: string }>) {
  return (
    <div className="grid gap-1">
      <span className="text-xs font-medium text-muted-foreground">{label}</span>
      <span className="break-all text-sm">{value}</span>
    </div>
  );
}

function SettingField({
  children,
  icon: Icon,
  label,
}: Readonly<{
  children: React.ReactNode;
  icon: typeof SunMoon;
  label: string;
}>) {
  return (
    <div className="flex min-w-0 flex-col items-start gap-2">
      <span className="flex h-5 items-center gap-2 text-sm font-medium">
        <Icon aria-hidden="true" className="size-4 text-muted-foreground" />
        {label}
      </span>
      {children}
    </div>
  );
}
