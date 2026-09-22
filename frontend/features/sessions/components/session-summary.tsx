import { CalendarDays, Clock3, UsersRound } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface SessionSummaryProps {
  startedAt: string;
  duration: string;
  participantCount: number;
}

export function SessionSummary({
  startedAt,
  duration,
  participantCount,
}: Readonly<SessionSummaryProps>) {
  const items = [
    { label: "Started", value: startedAt, icon: CalendarDays },
    { label: "Duration", value: duration, icon: Clock3 },
    { label: "Participants", value: String(participantCount), icon: UsersRound },
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Participation summary</CardTitle>
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
