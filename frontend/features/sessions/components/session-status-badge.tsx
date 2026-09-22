import { CircleDot } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import type { SessionStatus } from "./session-types";

export function SessionStatusBadge({
  status,
  locale = defaultLocale,
}: Readonly<{ status: SessionStatus; locale?: Locale }>) {
  const variant = status === "live" ? "default" : "secondary";
  const text = messagesFor(locale).sessions;
  const labels: Record<SessionStatus, string> = {
    scheduled: text.scheduled,
    live: text.liveNow,
    ended: text.ended,
  };

  return (
    <Badge className="gap-1.5" variant={variant}>
      {status === "live" ? <CircleDot className="size-3" aria-hidden="true" /> : null}
      {labels[status]}
    </Badge>
  );
}
