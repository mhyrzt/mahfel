import type { ReactNode } from "react";

import { AppShell } from "@/features/shell/components/app-shell";
import { getServerSessionUser } from "@/lib/auth/server";
import { getLocale } from "@/lib/i18n/server";

export default async function WorkspaceLayout({
  children,
  modal,
}: Readonly<{ children: ReactNode; modal: ReactNode }>) {
  const [locale, user] = await Promise.all([getLocale(), getServerSessionUser()]);
  return (
    <AppShell accountImage={user?.image} accountName={user?.name} locale={locale} modal={modal}>
      {children}
    </AppShell>
  );
}
