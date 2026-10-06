import type { ServerMessage } from "@/lib/websocket/generated/messages";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

export type LiveReaction = Extract<ServerMessage, { type: "session_reaction" }>;

export function ReactionOverlay({
  reactions,
  locale,
}: Readonly<{ reactions: LiveReaction[]; locale: Locale }>) {
  return (
    <div
      className="pointer-events-none absolute inset-0 z-20 overflow-hidden"
      aria-label={messagesFor(locale).interactions.reactions}
    >
      {reactions.map((reaction) => (
        <div
          key={reaction.event_id}
          data-session-reaction
          className="session-reaction-float absolute bottom-3 end-4 flex max-w-[min(20rem,80%)] items-center gap-2 rounded-full border bg-card/95 px-3 py-2 text-card-foreground shadow-sm"
          dir={locale === "fa" ? "rtl" : "ltr"}
        >
          <bdi className="truncate text-sm font-medium">{reaction.username}</bdi>
          <span className="text-3xl leading-none">{reaction.emoji}</span>
        </div>
      ))}
    </div>
  );
}
