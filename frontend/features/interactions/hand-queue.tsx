import { Hand } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { messagesFor, formatMessage } from "@/lib/i18n/messages";
import type { Locale } from "@/lib/i18n/config";
import type { RaisedHand, SendInteraction } from "./types";

export function HandQueue({
  hands,
  version,
  canModerate,
  busy,
  locale,
  send,
}: Readonly<{
  hands: RaisedHand[];
  version: number;
  canModerate: boolean;
  busy: boolean;
  locale: Locale;
  send: SendInteraction;
}>) {
  const text = messagesFor(locale).interactions;
  const numbers = new Intl.NumberFormat(locale);
  return (
    <section className="space-y-3 border-b px-6 pb-4" aria-label={text.handQueue}>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h3 className="flex items-center gap-2 text-sm font-semibold">
          <Hand className="size-4 text-warning" aria-hidden="true" />
          {text.handQueue}
          <Badge variant="secondary">{numbers.format(hands.length)}</Badge>
        </h3>
        {canModerate && hands.length > 0 ? (
          <Button
            size="sm"
            variant="ghost"
            disabled={busy}
            onClick={() => void send({ action: "clear_hands", expected_version: version })}
          >
            {text.clearHands}
          </Button>
        ) : null}
      </div>
      {hands.length === 0 ? (
        <p className="text-xs text-muted-foreground">{text.emptyHands}</p>
      ) : (
        <ol className="max-h-56 space-y-2 overflow-y-auto">
          {hands.map((hand, index) => (
            <li className="flex items-center justify-between gap-2" key={hand.participant_id}>
              <div className="min-w-0">
                <p className="truncate text-sm" dir="auto">
                  {hand.name}
                </p>
                <p className="text-xs text-muted-foreground">
                  {formatMessage(text.queuePosition, { number: numbers.format(index + 1) })}
                </p>
              </div>
              {canModerate ? (
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy}
                  aria-label={`${text.lowerHand}: ${hand.name}`}
                  onClick={() =>
                    void send({
                      action: "lower_hand",
                      participant_id: hand.participant_id,
                      expected_order: hand.order,
                    })
                  }
                >
                  <Hand className="size-3.5" aria-hidden="true" />
                  {text.lowerHand}
                </Button>
              ) : null}
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
