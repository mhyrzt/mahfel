import { Check, UserRoundCheck, X } from "lucide-react";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { formatMessage, messagesFor } from "@/lib/i18n/messages";

import type { AdmissionRequest } from "./session-types";

export function AdmissionQueue({
  requests,
  locale = defaultLocale,
}: Readonly<{ requests: AdmissionRequest[]; locale?: Locale }>) {
  const text = messagesFor(locale).sessions;

  return (
    <Card>
      <CardHeader className="flex-row items-center gap-2 space-y-0">
        <UserRoundCheck className="size-4" aria-hidden="true" />
        <CardTitle className="text-base">{text.admissionQueue}</CardTitle>
      </CardHeader>
      <CardContent>
        {requests.length === 0 ? (
          <p className="text-sm text-muted-foreground">{text.noAdmissionRequests}</p>
        ) : (
          <ul className="space-y-4">
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
                    {formatMessage(text.requested, { requestedAt: request.requestedAt })}
                  </p>
                </div>
                <Button
                  aria-label={formatMessage(text.admit, { name: request.name })}
                  size="icon"
                  variant="secondary"
                >
                  <Check className="size-4" />
                </Button>
                <Button
                  aria-label={formatMessage(text.reject, { name: request.name })}
                  size="icon"
                  variant="ghost"
                >
                  <X className="size-4" />
                </Button>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
