"use client";

import type { ReactNode } from "react";
import { SessionProvider } from "next-auth/react";

export function SessionRefresh({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <SessionProvider refetchInterval={180} refetchOnWindowFocus>
      {children}
    </SessionProvider>
  );
}
