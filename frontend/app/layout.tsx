import type { Metadata } from "next";
import { cookies } from "next/headers";

import { isTheme, themeCookieName } from "@/features/appearance/theme";
import { directionForLocale } from "@/lib/i18n/config";
import { getLocale } from "@/lib/i18n/server";

import "@fontsource-variable/vazirmatn";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    default: "Mahfel",
    template: "%s · Mahfel",
  },
  description: "Realtime communication and collaboration.",
};

export default async function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  const locale = await getLocale();
  const cookieStore = await cookies();
  const themeCookie = cookieStore.get(themeCookieName)?.value;
  const theme = isTheme(themeCookie) ? themeCookie : "system";

  return (
    <html data-theme={theme} dir={directionForLocale(locale)} lang={locale}>
      <body>{children}</body>
    </html>
  );
}
