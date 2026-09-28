"use client";

import { CalendarDays, Languages, SunMoon } from "lucide-react";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { DevicePreferences } from "@/features/account/components/device-preferences";
import { CalendarSwitcher } from "@/features/account/components/calendar-switcher";
import type { CalendarPreference } from "@/features/account/calendar-preference";
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
  const messages = messagesFor(locale).account;
  const initials = name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toLocaleUpperCase(locale))
    .join("");

  return (
    <div className="space-y-6">
      <header>
        <h1 className="text-xl font-semibold">{messages.title}</h1>
        <p className="mt-1 text-sm text-muted-foreground">{messages.description}</p>
      </header>
      <section className="grid gap-6 rounded-2xl border bg-card/80 p-5 sm:grid-cols-[minmax(9rem,0.55fr)_minmax(0,1.45fr)] sm:p-6">
        <div className="flex flex-col items-center gap-3 sm:items-start">
          <h2 className="text-sm font-medium">{messages.picture}</h2>
          <Avatar className="size-24 text-3xl ring-4 ring-primary/10">
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
      <section className="overflow-hidden rounded-2xl border bg-card/80">
        <h2 className="border-b px-5 py-4 text-base font-semibold sm:px-6">
          {messages.preferences}
        </h2>
        <div className="divide-y">
          <SettingField icon={SunMoon} label={messagesFor(locale).appearance.label}>
            <ThemeSwitcher locale={locale} theme={theme} />
          </SettingField>
          <SettingField icon={CalendarDays} label={messages.calendarType}>
            <CalendarSwitcher calendar={calendar} locale={locale} />
          </SettingField>
          <SettingField icon={Languages} label={messages.language}>
            <LocaleSwitcher locale={locale} />
          </SettingField>
        </div>
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
    <div className="grid min-w-0 gap-3 px-5 py-4 sm:grid-cols-[minmax(0,1fr)_minmax(17rem,1.2fr)] sm:items-center sm:gap-6 sm:px-6">
      <span className="flex min-w-0 items-center gap-3 text-sm font-medium">
        <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-muted/70">
          <Icon aria-hidden="true" className="size-4 text-muted-foreground" />
        </span>
        {label}
      </span>
      <div className="min-w-0">{children}</div>
    </div>
  );
}
