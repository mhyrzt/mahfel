"use client";

import { RouteError } from "@/features/shell/components/route-error";

import "./globals.css";

export default function GlobalError({
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <html lang="en">
      <body>
        <main className="min-h-dvh px-6">
          <RouteError scope="site" reset={reset} />
        </main>
      </body>
    </html>
  );
}
