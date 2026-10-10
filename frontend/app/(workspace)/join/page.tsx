import { redirect } from "next/navigation";

import { JoinByCodePanel } from "@/features/invites/components/join-by-code-panel";
import { getServerSessionUser } from "@/lib/auth/server";
import { getLocale } from "@/lib/i18n/server";

export default async function JoinPage({
  searchParams,
}: Readonly<{ searchParams: Promise<{ code?: string | string[] }> }>) {
  const query = await searchParams;
  const candidate = Array.isArray(query.code) ? query.code[0] : query.code;
  const initialCode = candidate && candidate.length <= 100 ? candidate.trim() : "";
  const [user, locale] = await Promise.all([getServerSessionUser(), getLocale()]);
  if (!user) {
    const destination = initialCode ? `/join?code=${encodeURIComponent(initialCode)}` : "/join";
    redirect(`/login?callbackUrl=${encodeURIComponent(destination)}`);
  }
  return <JoinByCodePanel locale={locale} initialCode={initialCode} />;
}
