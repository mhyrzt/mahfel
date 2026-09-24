import type { ReactNode } from "react";

import { AppShell } from "@/features/shell/components/app-shell";
import { getProfile } from "@/features/account/profile";
import { getServerSessionUser } from "@/lib/auth/server";
import { getLocale } from "@/lib/i18n/server";

export default async function WorkspaceLayout({
  children,
  modal,
}: Readonly<{ children: ReactNode; modal: ReactNode }>) {
  const [locale, user, profile] = await Promise.all([
    getLocale(),
    getServerSessionUser(),
    getProfile(),
  ]);
  return (
    <AppShell
      accountImage={profile?.avatar_url ?? user?.image}
      accountName={profile?.display_name ?? user?.name}
      locale={locale}
      modal={modal}
    >
      {children}
    </AppShell>
  );
}
