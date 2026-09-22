import { Mic, MicOff, UsersRound } from "lucide-react";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import type { Participant } from "./session-types";

export function ParticipantRoster({
  participants,
  locale = defaultLocale,
}: Readonly<{ participants: Participant[]; locale?: Locale }>) {
  const text = messagesFor(locale).sessions;

  return (
    <Card className="h-full shadow-none">
      <CardHeader className="flex-row items-center justify-between space-y-0">
        <CardTitle className="flex items-center gap-2 text-base">
          <UsersRound className="size-4" aria-hidden="true" />
          {text.participants}
        </CardTitle>
        <Badge variant="secondary">{participants.length}</Badge>
      </CardHeader>
      <CardContent className="h-80 p-0">
        {participants.length === 0 ? (
          <div className="flex h-full flex-col items-center justify-center gap-2 px-6 text-center text-sm text-muted-foreground">
            <UsersRound className="size-6" aria-hidden="true" />
            {text.emptyRoster}
          </div>
        ) : (
          <ScrollArea className="h-full px-6 pb-6">
            <ul className="space-y-3">
              {participants.map((participant) => (
                <li className="flex items-center justify-between gap-3" key={participant.id}>
                  <div className="flex min-w-0 items-center gap-3">
                    <Avatar className="size-9">
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
                      className="size-4 shrink-0 text-muted-foreground"
                      aria-label={text.microphoneOn}
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
