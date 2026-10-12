"use client";

import { Languages } from "lucide-react";
import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { locales, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import { persistLocalePreference } from "@/features/i18n/locale-preference";

interface LocaleSwitcherProps {
  locale: Locale;
  compact?: boolean;
}

export function LocaleSwitcher({ locale, compact = false }: Readonly<LocaleSwitcherProps>) {
  const router = useRouter();
  const messages = messagesFor(locale);
  const options = locales.map((value) => ({ value, label: messages.language[value] }));

  function changeLocale(nextLocale: Locale) {
    persistLocalePreference(nextLocale);
    router.refresh();
  }

  if (compact) {
    return (
      <DropdownMenu dir={locale === "fa" ? "rtl" : "ltr"}>
        <DropdownMenuTrigger asChild>
          <Button
            aria-label={messages.account.language}
            title={messages.account.language}
            className="h-11 gap-2 rounded-xl border-border/70 bg-transparent px-3.5 text-xs font-semibold shadow-none"
            variant="outline"
          >
            <Languages aria-hidden="true" className="size-4" />
            <span>{locale === "fa" ? "فا" : "EN"}</span>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="min-w-36 rounded-xl p-1.5">
          <DropdownMenuRadioGroup
            onValueChange={(value) => {
              if (value === "en" || value === "fa") {
                changeLocale(value);
              }
            }}
            value={locale}
          >
            {options.map(({ value, label }) => (
              <DropdownMenuRadioItem className="rounded-lg" key={value} value={value}>
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
      label={messages.account.language}
      onValueChange={changeLocale}
      options={options}
      value={locale}
    />
  );
}
