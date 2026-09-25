"use client";

import { RouteError } from "@/features/shell/components/route-error";

export default function Error({
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return <RouteError scope="workspace" reset={reset} />;
}
