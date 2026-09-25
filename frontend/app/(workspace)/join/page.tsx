import { redirect } from "next/navigation";

import { JoinByCodePanel } from "@/features/invites/components/join-by-code-panel";
import { getServerSessionUser } from "@/lib/auth/server";
import { getLocale } from "@/lib/i18n/server";

export default async function JoinPage() {
  const [user, locale] = await Promise.all([getServerSessionUser(), getLocale()]);
  if (!user) {
    redirect("/login?callbackUrl=%2Fjoin");
  }
  return <JoinByCodePanel locale={locale} />;
}
