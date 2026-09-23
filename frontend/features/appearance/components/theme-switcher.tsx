"use client";

import { useRouter } from "next/navigation";

import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import { isTheme, themeCookieName, themes, type Theme } from "../theme";

interface ThemeSwitcherProps {
  locale: Locale;
  theme: Theme;
}

export function ThemeSwitcher({ locale, theme }: Readonly<ThemeSwitcherProps>) {
  const router = useRouter();
  const messages = messagesFor(locale).appearance;

  function changeTheme(nextTheme: Theme) {
    document.cookie = `${themeCookieName}=${nextTheme}; Path=/; Max-Age=31536000; SameSite=Lax`;
    document.documentElement.dataset.theme = nextTheme;
    router.refresh();
  }

  return (
    <label className="flex items-center gap-2">
      <span className="sr-only">{messages.label}</span>
      <select
        aria-label={messages.label}
        className="h-10 rounded-md border border-input bg-background px-3 text-sm text-foreground shadow-none outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        onChange={(event) => {
          const value = event.currentTarget.value;
          if (isTheme(value)) {
            changeTheme(value);
          }
        }}
        value={theme}
      >
        {themes.map((availableTheme) => (
          <option key={availableTheme} value={availableTheme}>
            {messages[availableTheme]}
          </option>
        ))}
      </select>
    </label>
  );
}
