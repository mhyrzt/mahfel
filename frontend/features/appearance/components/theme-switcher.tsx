"use client";

import { useRouter } from "next/navigation";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
    <Select
      onValueChange={(value) => {
        if (isTheme(value)) {
          changeTheme(value);
        }
      }}
      value={theme}
    >
      <SelectTrigger aria-label={messages.label}>
        <SelectValue />
      </SelectTrigger>
      <SelectContent dir={locale === "fa" ? "rtl" : "ltr"}>
        {themes.map((availableTheme) => (
          <SelectItem key={availableTheme} value={availableTheme}>
            {messages[availableTheme]}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
