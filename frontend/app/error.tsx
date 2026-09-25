"use client";

import { RouteError } from "@/features/shell/components/route-error";

export default function Error({
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <main className="min-h-dvh px-6">
      <RouteError scope="site" reset={reset} />
    </main>
  );
}
