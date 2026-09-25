"use client";

import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/button";
import { locales, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import { persistLocalePreference } from "@/features/i18n/locale-preference";

interface LocaleSwitcherProps {
  locale: Locale;
}

export function LocaleSwitcher({ locale }: Readonly<LocaleSwitcherProps>) {
  const router = useRouter();
  const messages = messagesFor(locale);

  function changeLocale(nextLocale: Locale) {
    persistLocalePreference(nextLocale);
    router.refresh();
  }

  return (
    <div className="grid w-full grid-cols-2 gap-1" aria-label="Language">
      {locales.map((availableLocale) => (
        <Button
          aria-pressed={availableLocale === locale}
          key={availableLocale}
          onClick={() => changeLocale(availableLocale)}
          className="h-10"
          variant={availableLocale === locale ? "secondary" : "ghost"}
        >
          {messages.language[availableLocale]}
        </Button>
      ))}
    </div>
  );
}
