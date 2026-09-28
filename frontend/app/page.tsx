import { cookies } from "next/headers";

import { LandingPageView } from "@/features/landing/components/landing-page-view";
import { isTheme, themeCookieName } from "@/features/appearance/theme";
import { getLocale } from "@/lib/i18n/server";

export default async function Home() {
  const locale = await getLocale();
  const cookieStore = await cookies();
  const themeCookie = cookieStore.get(themeCookieName)?.value;
  const theme = isTheme(themeCookie) ? themeCookie : "system";

  return <LandingPageView locale={locale} theme={theme} />;
}
