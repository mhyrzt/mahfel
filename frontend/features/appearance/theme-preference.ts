"use client";

import { themeCookieName, type Theme } from "./theme";

export function persistThemePreference(theme: Theme) {
  document.cookie = `${themeCookieName}=${theme}; Path=/; Max-Age=31536000; SameSite=Lax`;
  document.documentElement.dataset.theme = theme;
}
