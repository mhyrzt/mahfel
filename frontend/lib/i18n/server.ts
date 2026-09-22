import "server-only";

import { cookies } from "next/headers";

import { defaultLocale, isLocale, localeCookieName, type Locale } from "./config";

export async function getLocale(): Promise<Locale> {
  const locale = (await cookies()).get(localeCookieName)?.value;
  return isLocale(locale) ? locale : defaultLocale;
}
