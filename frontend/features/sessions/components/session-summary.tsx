import { CalendarDays, Clock3, UsersRound } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

interface SessionSummaryProps {
  startedAt: string;
  duration: string;
  participantCount: number;
  locale?: Locale;
}

export function SessionSummary({
  startedAt,
  duration,
  participantCount,
  locale = defaultLocale,
}: Readonly<SessionSummaryProps>) {
  const text = messagesFor(locale).sessions;
  const items = [
    { label: text.started, value: startedAt, icon: CalendarDays },
    { label: text.duration, value: duration, icon: Clock3 },
    { label: text.participants, value: String(participantCount), icon: UsersRound },
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{text.participationSummary}</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-4 sm:grid-cols-3">
        {items.map(({ label, value, icon: Icon }) => (
          <div className="flex items-center gap-3" key={label}>
            <Icon className="size-4 text-muted-foreground" aria-hidden="true" />
            <div>
              <p className="text-xs text-muted-foreground">{label}</p>
              <p className="text-sm font-medium">{value}</p>
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
