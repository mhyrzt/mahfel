import { useState } from "react";
import { HandControl } from "@/features/interactions/hand-control";
import { ReactionControl } from "@/features/interactions/reaction-control";
import { ReactionOverlay } from "@/features/interactions/reaction-overlay";
import { useSessionReactions } from "@/features/interactions/use-session-reactions";
import type { Locale } from "@/lib/i18n/config";

export function useInteractionPreview(locale: Locale, disabled = false, initialRaised = false) {
  const [raised, setRaised] = useState(initialRaised);
  const reactions = useSessionReactions("session-demo", locale, !disabled);
  const toggleHand = () => setRaised((value) => !value);
  return {
    handControl: (
      <HandControl locale={locale} raised={raised} disabled={disabled} onToggle={toggleHand} />
    ),
    reactionControl: (
      <ReactionControl
        locale={locale}
        disabled={disabled}
        handRaised={raised}
        onToggleHand={toggleHand}
        onReact={(emoji) =>
          reactions.handleMessage({
            v: 1,
            type: "session_reaction",
            session_id: "session-demo",
            event_id: crypto.randomUUID(),
            participant_id: "preview-participant",
            username: locale === "fa" ? "سارا" : "Sara",
            emoji,
          })
        }
      />
    ),
    reactionOverlay: <ReactionOverlay locale={locale} reactions={reactions.reactions} />,
  };
}
