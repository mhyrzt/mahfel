import type { ReactNode } from "react";

import { AppShell } from "@/features/shell/components/app-shell";

export default function WorkspaceLayout({ children }: Readonly<{ children: ReactNode }>) {
  return <AppShell>{children}</AppShell>;
}
