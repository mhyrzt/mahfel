import { Clock3 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { defaultLocale, type Locale } from "@/lib/i18n/config";
import { formatMessage, messagesFor } from "@/lib/i18n/messages";

export function WaitingRoomPanel({
  roomName,
  locale = defaultLocale,
  onRetry,
}: Readonly<{ roomName: string; locale?: Locale; onRetry?: () => void }>) {
  const text = messagesFor(locale).sessions;

  return (
    <Card className="mx-auto max-w-lg">
      <CardHeader className="items-center text-center">
        <div className="mb-2 rounded-full bg-muted p-3">
          <Clock3 className="size-6 text-muted-foreground" aria-hidden="true" />
        </div>
        <CardTitle>{text.waitingTitle}</CardTitle>
        <CardDescription>{formatMessage(text.waitingDescription, { roomName })}</CardDescription>
      </CardHeader>
      <CardContent>
        <p className="rounded-lg bg-muted px-3 py-2 text-center text-sm text-muted-foreground">
          {text.waitingHint}
        </p>
        {onRetry ? (
          <Button className="mt-4 w-full" variant="outline" onClick={onRetry}>
            {text.checkAdmission}
          </Button>
        ) : null}
      </CardContent>
    </Card>
  );
}
