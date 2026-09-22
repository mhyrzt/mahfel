import type { ReactNode } from "react";

export default function AuthLayout({ children }: Readonly<{ children: ReactNode }>) {
  return <div className="min-h-screen bg-muted/40">{children}</div>;
}
