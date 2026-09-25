import { Check, UserRoundCheck, X } from "lucide-react";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { formatMessage, messagesFor } from "@/lib/i18n/messages";

import type { AdmissionRequest } from "./session-types";

export function AdmissionQueue({
  requests,
  locale = defaultLocale,
  onAdmit,
  onReject,
}: Readonly<{
  requests: AdmissionRequest[];
  locale?: Locale;
  onAdmit?: (request: AdmissionRequest) => void;
  onReject?: (request: AdmissionRequest) => void;
}>) {
  const text = messagesFor(locale).sessions;

  return (
    <Card className="flex h-full flex-col">
      <CardHeader className="flex-row items-center gap-2 space-y-0">
        <UserRoundCheck className="size-4" aria-hidden="true" />
        <CardTitle className="text-base">{text.admissionQueue}</CardTitle>
      </CardHeader>
      <CardContent className="min-h-0 flex-1 overflow-hidden">
        {requests.length === 0 ? (
          <p className="text-sm text-muted-foreground">{text.noAdmissionRequests}</p>
        ) : (
          <ScrollArea className="h-full" dir={locale === "fa" ? "rtl" : "ltr"}>
            <ul className="space-y-4 pb-4">
              {requests.map((request) => (
                <li className="flex items-center gap-3" key={request.id}>
                  <Avatar className="size-9">
                    <AvatarFallback className="text-xs font-semibold">
                      {request.initials}
                    </AvatarFallback>
                  </Avatar>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{request.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {formatElapsedTime(request.requestedAt, locale)}
                    </p>
                  </div>
                  <Button
                    aria-label={formatMessage(text.admit, { name: request.name })}
                    disabled={!request.actorId || !onAdmit}
                    onClick={() => onAdmit?.(request)}
                    size="icon"
                    variant="secondary"
                  >
                    <Check className="size-4" />
                  </Button>
                  <Button
                    aria-label={formatMessage(text.reject, { name: request.name })}
                    disabled={!request.actorId || !onReject}
                    onClick={() => onReject?.(request)}
                    size="icon"
                    variant="ghost"
                  >
                    <X className="size-4" />
                  </Button>
                </li>
              ))}
            </ul>
          </ScrollArea>
        )}
      </CardContent>
    </Card>
  );
}

function formatElapsedTime(requestedAt: string, locale: Locale) {
  const elapsedMinutes = Math.max(
    0,
    Math.floor((Date.now() - new Date(requestedAt).getTime()) / 60_000),
  );
  let value = elapsedMinutes;
  let unit: Intl.RelativeTimeFormatUnit = "minute";

  if (elapsedMinutes >= 1_440) {
    value = Math.floor(elapsedMinutes / 1_440);
    unit = "day";
  } else if (elapsedMinutes >= 60) {
    value = Math.floor(elapsedMinutes / 60);
    unit = "hour";
  }

  const language = locale === "fa" ? "fa-IR" : "en-US";
  return new Intl.RelativeTimeFormat(language, { numeric: "auto" }).format(-value, unit);
}
