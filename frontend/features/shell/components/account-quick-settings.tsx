"use client";

import { signOut } from "next-auth/react";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState } from "react";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { persistCalendarPreference } from "@/features/account/calendar-preference-client";
import { AccountSettingsPanel } from "@/features/account/components/account-settings-panel";
import {
  calendarPreferences,
  type CalendarPreference,
} from "@/features/account/calendar-preference";
import { persistThemePreference } from "@/features/appearance/theme-preference";
import { themes, type Theme } from "@/features/appearance/theme";
import { persistLocalePreference } from "@/features/i18n/locale-preference";
import { locales, type Locale } from "@/lib/i18n/config";
import { getFederatedLogoutUrl } from "@/lib/auth/federated-logout-client";
import { messagesFor } from "@/lib/i18n/messages";

interface AccountQuickSettingsProps {
  accountImage?: string | null;
  accountName?: string | null;
  calendar: CalendarPreference;
  email?: string;
  locale: Locale;
  theme: Theme;
}

export function AccountQuickSettings({
  accountImage,
  accountName,
  calendar,
  email = "",
  locale,
  theme,
}: Readonly<AccountQuickSettingsProps>) {
  const [open, setOpen] = useState(false);
  const [logoutError, setLogoutError] = useState(false);
  const [loggingOut, setLoggingOut] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const popupRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const router = useRouter();
  const messages = messagesFor(locale);
  const shell = messages.shell;
  const accountInitial = accountName?.trim().slice(0, 1).toLocaleUpperCase(locale) || "?";

  useEffect(() => {
    function closeOnOutsidePointer(event: PointerEvent) {
      if (
        event.target instanceof Node &&
        !popupRef.current?.contains(event.target) &&
        !triggerRef.current?.contains(event.target)
      ) {
        setOpen(false);
      }
    }

    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setOpen(false);
        triggerRef.current?.focus();
      }
    }

    if (open) {
      document.addEventListener("pointerdown", closeOnOutsidePointer);
      document.addEventListener("keydown", closeOnEscape);
    }
    return () => {
      document.removeEventListener("pointerdown", closeOnOutsidePointer);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, [open]);

  function changeCalendar(value: CalendarPreference) {
    persistCalendarPreference(value);
    router.refresh();
  }

  function changeTheme(value: Theme) {
    persistThemePreference(value);
    router.refresh();
  }

  function changeLocale(value: Locale) {
    persistLocalePreference(value);
    router.refresh();
  }

  async function logout() {
    setLoggingOut(true);
    setLogoutError(false);
    try {
      const logoutUrl = await getFederatedLogoutUrl();
      await signOut({ redirect: false, callbackUrl: "/login?loggedOut=1" });
      window.location.assign(logoutUrl);
    } catch {
      setLoggingOut(false);
      setLogoutError(true);
    }
  }

  return (
    <div
      className="group relative"
      onMouseEnter={() => setOpen(true)}
      onMouseLeave={(event) => {
        if (
          !(event.relatedTarget instanceof Node) ||
          !event.currentTarget.contains(event.relatedTarget)
        ) {
          setOpen(false);
        }
      }}
      onFocus={() => setOpen(true)}
    >
      <Button
        aria-label={messages.account.openSettings}
        aria-expanded={open}
        aria-controls="account-quick-settings"
        className="size-12 rounded-full"
        onClick={() => {
          setOpen(false);
          setSettingsOpen(true);
        }}
        ref={triggerRef}
        variant="ghost"
      >
        <Avatar className="size-8">
          <AvatarImage alt="" src={accountImage ?? undefined} />
          <AvatarFallback className="bg-primary text-xs font-semibold text-primary-foreground">
            {accountInitial}
          </AvatarFallback>
        </Avatar>
      </Button>
      {open && (
        <div
          id="account-quick-settings"
          ref={popupRef}
          className="absolute bottom-0 start-full z-50 w-72 rounded-lg border bg-popover p-3 text-popover-foreground shadow-lg"
          dir={locale === "fa" ? "rtl" : "ltr"}
        >
          <Button
            className="mb-3 w-full justify-start px-2"
            onClick={() => {
              setOpen(false);
              setSettingsOpen(true);
            }}
            variant="ghost"
          >
            {messages.account.openSettings}
          </Button>
          <div className="space-y-3 border-t pt-3">
            <PreferenceGroup label={messages.appearance.label}>
              {themes.map((value) => (
                <Button
                  aria-pressed={value === theme}
                  className="h-9 px-2 text-xs"
                  key={value}
                  onClick={() => changeTheme(value)}
                  variant={theme === value ? "secondary" : "ghost"}
                >
                  {messages.appearance[value]}
                </Button>
              ))}
            </PreferenceGroup>
            <PreferenceGroup label={messages.account.calendarType}>
              {calendarPreferences.map((value) => (
                <Button
                  aria-pressed={value === calendar}
                  className="h-9 px-2 text-xs"
                  key={value}
                  onClick={() => changeCalendar(value)}
                  variant={calendar === value ? "secondary" : "ghost"}
                >
                  {messages.account[value]}
                </Button>
              ))}
            </PreferenceGroup>
            <PreferenceGroup label={messages.account.language}>
              {locales.map((value) => (
                <Button
                  aria-pressed={value === locale}
                  className="h-9 px-2 text-xs"
                  key={value}
                  onClick={() => changeLocale(value)}
                  variant={locale === value ? "secondary" : "ghost"}
                >
                  {messages.language[value]}
                </Button>
              ))}
            </PreferenceGroup>
          </div>
          {logoutError && (
            <p className="mt-3 text-sm text-destructive" role="alert">
              {shell.logoutFailed}
            </p>
          )}
          <Button
            className="mt-3 w-full justify-start"
            disabled={loggingOut}
            onClick={logout}
            variant="ghost"
          >
            {shell.logout}
          </Button>
        </div>
      )}
      <Dialog onOpenChange={setSettingsOpen} open={settingsOpen}>
        <DialogContent className="max-h-[calc(100dvh-2rem)] max-w-4xl overflow-y-auto p-5 sm:p-7">
          <DialogTitle className="sr-only">{messages.account.title}</DialogTitle>
          <DialogDescription className="sr-only">{messages.account.description}</DialogDescription>
          <AccountSettingsPanel
            calendar={calendar}
            email={email}
            image={accountImage ?? null}
            locale={locale}
            name={accountName ?? ""}
            theme={theme}
          />
        </DialogContent>
      </Dialog>
    </div>
  );
}

function PreferenceGroup({
  children,
  label,
}: Readonly<{ children: React.ReactNode; label: string }>) {
  return (
    <fieldset className="space-y-1">
      <legend className="px-2 text-xs font-medium text-muted-foreground">{label}</legend>
      <div className="grid grid-flow-col auto-cols-fr gap-1">{children}</div>
    </fieldset>
  );
}
