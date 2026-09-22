"use client";

import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/button";
import { localeCookieName, locales, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

function persistLocalePreference(locale: Locale) {
  document.cookie = `${localeCookieName}=${locale}; Path=/; Max-Age=31536000; SameSite=Lax`;
}

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
    <div className="flex items-center gap-1" aria-label="Language">
      {locales.map((availableLocale) => (
        <Button
          aria-pressed={availableLocale === locale}
          key={availableLocale}
          onClick={() => changeLocale(availableLocale)}
          size="sm"
          variant={availableLocale === locale ? "secondary" : "ghost"}
        >
          {messages.language[availableLocale]}
        </Button>
      ))}
    </div>
  );
}
