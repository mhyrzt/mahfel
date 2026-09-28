import { useState } from "react";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";
import { ConversationPanel } from "../components/conversation-panel";
import { ChatMessageList } from "../components/chat-message-list";

export function ConversationPreview({
  locale = "en",
  withAttachment = true,
}: Readonly<{ locale?: Locale; withAttachment?: boolean }>) {
  const [draft, setDraft] = useState("");
  const [attachments, setAttachments] = useState<File[]>(() =>
    withAttachment
      ? [
          new File(["agenda"], `${locale === "fa" ? "دستور-جلسه" : "agenda"}.pdf`, {
            type: "application/pdf",
          }),
        ]
      : [],
  );
  const [messages, setMessages] = useState<components["schemas"]["ChatMessageResponse"][]>([
    {
      id: "preview-message",
      channel_id: "preview-room",
      author_id: "member-1",
      content: locale === "fa" ? "یادداشت‌های کارگاه آماده است." : "The workshop notes are ready.",
      created_at: "2026-09-28T09:00:00Z",
    },
  ]);
  return (
    <ConversationPanel
      compact
      roomId="preview-room"
      locale={locale}
      draft={draft}
      onDraftChange={setDraft}
      onSend={(event) => {
        event.preventDefault();
        if (!draft.trim()) {
          return;
        }
        setMessages((previous) => [
          ...previous,
          {
            id: `preview-${previous.length}`,
            channel_id: "preview-room",
            author_id: "manager-1",
            content: draft.trim(),
            created_at: new Date().toISOString(),
          },
        ]);
        setDraft("");
        setAttachments([]);
      }}
      attachments={attachments}
      onAttachFiles={(files) => setAttachments((previous) => [...previous, ...files])}
      onRemoveAttachment={(index) =>
        setAttachments((previous) => previous.filter((_, position) => position !== index))
      }
    >
      <ChatMessageList
        messages={messages}
        roomId="preview-room"
        locale={locale}
        actorId="manager-1"
      />
    </ConversationPanel>
  );
}
