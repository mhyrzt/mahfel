import { CircleDot } from "lucide-react";

import { Badge } from "@/components/ui/badge";

import type { SessionStatus } from "./session-types";

const labels: Record<SessionStatus, string> = {
  scheduled: "Scheduled",
  live: "Live now",
  ended: "Ended",
};

export function SessionStatusBadge({ status }: Readonly<{ status: SessionStatus }>) {
  const variant = status === "live" ? "default" : "secondary";

  return (
    <Badge className="gap-1.5" variant={variant}>
      {status === "live" ? <CircleDot className="size-3" aria-hidden="true" /> : null}
      {labels[status]}
    </Badge>
  );
}
