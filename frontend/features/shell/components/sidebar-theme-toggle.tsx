"use client";

import { useState, useSyncExternalStore } from "react";
import { Moon, Sun } from "lucide-react";
import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { persistThemePreference } from "@/features/appearance/theme-preference";
import type { Theme } from "@/features/appearance/theme";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

const mediaQuery = "(prefers-color-scheme: dark)";
const subscribe = (callback: () => void) => {
  if (!window.matchMedia) {
    return () => {};
  }
  const query = window.matchMedia(mediaQuery);
  query.addEventListener("change", callback);
  return () => query.removeEventListener("change", callback);
};
const systemIsDark = () => window.matchMedia?.(mediaQuery).matches ?? false;
const serverIsDark = () => false;

export function SidebarThemeToggle({
  initialTheme,
  locale,
}: Readonly<{ initialTheme: Theme; locale: Locale }>) {
  const [theme, setTheme] = useState(initialTheme);
  const router = useRouter();
  const prefersDark = useSyncExternalStore(subscribe, systemIsDark, serverIsDark);
  const dark = theme === "dark" || (theme === "system" && prefersDark);
  const next = dark ? "light" : "dark";
  const label = messagesFor(locale).appearance[next];

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            aria-label={label}
            className="size-11 rounded-xl"
            onClick={() => {
              persistThemePreference(next);
              setTheme(next);
              router.refresh();
            }}
            size="icon"
            variant="ghost"
          >
            {dark ? (
              <Sun className="size-5" aria-hidden="true" />
            ) : (
              <Moon className="size-5" aria-hidden="true" />
            )}
          </Button>
        </TooltipTrigger>
        <TooltipContent side={locale === "fa" ? "left" : "right"}>{label}</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
