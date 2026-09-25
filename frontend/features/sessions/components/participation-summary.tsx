import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import { roomCopy } from "@/features/rooms/room-copy";

type Participation = components["schemas"]["SessionParticipationResponse"];

export function ParticipationSummary({
  data,
  locale,
}: Readonly<{ data: Participation | null; locale: Locale }>) {
  const text = messagesFor(locale).sessions;
  const roomText = roomCopy(locale);
  const language = locale === "fa" ? "fa-IR" : "en-US";
  return (
    <section
      aria-label={text.participationSummary}
      className="space-y-3"
      data-testid="participation-summary"
    >
      <h2 className="text-lg font-semibold">{text.participationSummary}</h2>
      {!data ? (
        <p className="text-sm text-muted-foreground">{text.sessionTimingUnavailable}</p>
      ) : null}
      {data?.participants.length === 0 ? (
        <p className="text-sm text-muted-foreground">{roomText.noParticipants}</p>
      ) : null}
      {data && data.participants.length > 0 ? (
        <ul className="divide-y border-y">
          {data.participants.map((participant) => (
            <li
              className="flex items-center justify-between gap-4 py-3 text-sm"
              key={participant.participant_id}
            >
              <span>
                {roomText.participant} {participant.actor_id.slice(0, 8)}
              </span>
              <span>
                {new Intl.NumberFormat(language, {
                  style: "unit",
                  unit: "minute",
                  maximumFractionDigits: 1,
                }).format(participant.connected_seconds / 60)}
              </span>
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
