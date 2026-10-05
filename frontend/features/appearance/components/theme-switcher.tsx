"use client";

import { Monitor, Moon, Sun } from "lucide-react";
import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { SegmentedControl, type SegmentOption } from "@/components/ui/segmented-control";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import { isTheme, themes, type Theme } from "../theme";
import { persistThemePreference } from "../theme-preference";

interface ThemeSwitcherProps {
  locale: Locale;
  theme: Theme;
  compact?: boolean;
}

const icons = { system: Monitor, light: Sun, dark: Moon } as const;

export function ThemeSwitcher({ locale, theme, compact = false }: Readonly<ThemeSwitcherProps>) {
  const router = useRouter();
  const messages = messagesFor(locale).appearance;
  const options: SegmentOption<Theme>[] = themes.map((value) => ({
    value,
    label: messages[value],
    icon: icons[value],
  }));
  const ActiveIcon = icons[theme];

  function changeTheme(nextTheme: Theme) {
    persistThemePreference(nextTheme);
    router.refresh();
  }

  if (compact) {
    return (
      <DropdownMenu dir={locale === "fa" ? "rtl" : "ltr"}>
        <DropdownMenuTrigger asChild>
          <Button
            aria-label={messages.label}
            className="size-9 shrink-0 rounded-lg border bg-background/75 shadow-none"
            size="icon"
            variant="outline"
          >
            <ActiveIcon aria-hidden="true" className="size-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="min-w-36 rounded-xl p-1.5">
          <DropdownMenuRadioGroup
            onValueChange={(value) => {
              if (isTheme(value)) {
                changeTheme(value);
              }
            }}
            value={theme}
          >
            {options.map(({ value, label, icon: Icon }) => (
              <DropdownMenuRadioItem className="gap-2 rounded-lg" key={value} value={value}>
                {Icon ? <Icon aria-hidden="true" className="size-4" /> : null}
                {label}
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>
        </DropdownMenuContent>
      </DropdownMenu>
    );
  }

  return (
    <SegmentedControl
      dir={locale === "fa" ? "rtl" : "ltr"}
      label={messages.label}
      onValueChange={changeTheme}
      options={options}
      value={theme}
    />
  );
}
