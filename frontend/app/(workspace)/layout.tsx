import type { ReactNode } from "react";

import { AppShell } from "@/features/shell/components/app-shell";
import { getLocale } from "@/lib/i18n/server";

export default async function WorkspaceLayout({ children }: Readonly<{ children: ReactNode }>) {
  const locale = await getLocale();
  return <AppShell locale={locale}>{children}</AppShell>;
}
