"use client";

import { localeCookieName, type Locale } from "@/lib/i18n/config";

export function persistLocalePreference(locale: Locale) {
  document.cookie = `${localeCookieName}=${locale}; Path=/; Max-Age=31536000; SameSite=Lax`;
}
