import { Mic, MicOff, UsersRound } from "lucide-react";
import type { ReactNode } from "react";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import type { Participant } from "./session-types";

const roleOrder: Record<Participant["role"], number> = { manager: 0, moderator: 1, member: 2 };

export function ParticipantRoster({
  participants,
  summary,
  locale = defaultLocale,
}: Readonly<{ participants: Participant[]; summary?: ReactNode; locale?: Locale }>) {
  const text = messagesFor(locale).sessions;
  const names = new Intl.Collator(locale, { sensitivity: "base", numeric: true });

  return (
    <Card className="flex h-full flex-col shadow-none">
      <CardHeader className="flex-row items-center justify-between space-y-0">
        <CardTitle className="flex items-center gap-2 text-base">
          <UsersRound className="size-4" aria-hidden="true" />
          {text.participants}
        </CardTitle>
        <Badge variant="secondary">{participants.length}</Badge>
      </CardHeader>
      {summary}
      <CardContent className="min-h-0 flex-1 p-0">
        {participants.length === 0 ? (
          <div className="flex h-full flex-col items-center justify-center gap-2 px-6 text-center text-sm text-muted-foreground">
            <UsersRound className="size-6" aria-hidden="true" />
            {text.emptyRoster}
          </div>
        ) : (
          <ScrollArea className="h-full px-6 pb-6" dir={locale === "fa" ? "rtl" : "ltr"}>
            <ul className="space-y-2">
              {[...participants]
                .sort(
                  (left, right) =>
                    roleOrder[left.role] - roleOrder[right.role] ||
                    Number(Boolean(right.speaking)) - Number(Boolean(left.speaking)) ||
                    names.compare(left.name, right.name),
                )
                .map((participant) => (
                  <li
                    className={`flex items-center justify-between gap-3 rounded-lg px-2 py-2 transition-colors ${participant.speaking ? "bg-success/10" : ""}`}
                    key={participant.id}
                  >
                    <div className="flex min-w-0 items-center gap-3">
                      <Avatar
                        className={`size-9 ${participant.speaking ? "ring-2 ring-success ring-offset-2 ring-offset-background" : ""}`}
                      >
                        <AvatarFallback className="text-xs font-semibold">
                          {participant.initials}
                        </AvatarFallback>
                      </Avatar>
                      <div className="min-w-0">
                        <p className="truncate text-sm font-medium">{participant.name}</p>
                        <p className="text-xs text-muted-foreground">
                          {text.roles[participant.role]}
                        </p>
                      </div>
                    </div>
                    {participant.muted ? (
                      <MicOff
                        className="size-4 shrink-0 text-muted-foreground"
                        aria-label={text.muted}
                      />
                    ) : (
                      <Mic
                        className={`size-4 shrink-0 ${participant.speaking ? "text-success drop-shadow-[0_0_8px_var(--success)]" : "text-muted-foreground"}`}
                        aria-label={participant.speaking ? text.speaking : text.microphoneOn}
                      />
                    )}
                  </li>
                ))}
            </ul>
          </ScrollArea>
        )}
      </CardContent>
    </Card>
  );
}
