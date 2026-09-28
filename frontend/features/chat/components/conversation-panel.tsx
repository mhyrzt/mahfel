"use client";

import type { FormEvent, ReactNode } from "react";
import { Paperclip, Send, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { roomCopy } from "@/features/rooms/room-copy";
import type { Locale } from "@/lib/i18n/config";

export function ConversationPanel({
  roomId,
  locale,
  compact = false,
  draft,
  onDraftChange,
  onSend,
  sending = false,
  replying = false,
  onCancelReply,
  attachments = [],
  onAttachFiles,
  onRemoveAttachment,
  error,
  children,
}: Readonly<{
  roomId: string;
  locale: Locale;
  compact?: boolean;
  draft: string;
  onDraftChange: (draft: string) => void;
  onSend: (event: FormEvent<HTMLFormElement>) => void;
  sending?: boolean;
  replying?: boolean;
  onCancelReply?: () => void;
  attachments?: File[];
  onAttachFiles?: (files: File[]) => void;
  onRemoveAttachment?: (index: number) => void;
  error?: string | null;
  children: ReactNode;
}>) {
  const text = roomCopy(locale);
  const attachable = Boolean(onAttachFiles);
  return (
    <section
      aria-label={text.chat}
      data-testid="room-chat"
      className={
        compact
          ? "flex h-full min-h-0 flex-col gap-3 bg-sidebar p-4"
          : "flex min-h-[28rem] flex-col gap-4 rounded-2xl border bg-card p-4 sm:p-6"
      }
    >
      {!compact ? (
        <div className="border-b pb-3">
          <p className="mb-1 text-xs font-semibold tracking-wide text-muted-foreground">
            {text.roomActivity}
          </p>
          <h2 className="text-xl font-semibold tracking-tight">{text.chat}</h2>
        </div>
      ) : null}
      {error ? (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      ) : null}
      {children}
      <form className="mt-auto shrink-0 rounded-xl border bg-background p-2" onSubmit={onSend}>
        {replying ? (
          <div className="mb-2 flex items-center justify-between border-s-2 border-primary px-2 text-xs text-muted-foreground">
            {text.reply}
            <Button onClick={onCancelReply} size="sm" type="button" variant="ghost">
              {text.cancel}
            </Button>
          </div>
        ) : null}
        {attachments.length > 0 ? (
          <ul className="mb-2 flex flex-wrap gap-1.5" data-testid="chat-attachments">
            {attachments.map((file, index) => (
              <li
                className="flex max-w-full items-center gap-1 rounded-md bg-muted px-2 py-1 text-xs"
                key={`${file.name}-${file.size}-${index}`}
              >
                <Paperclip aria-hidden="true" className="size-3 shrink-0" />
                <span className="min-w-0 truncate">{file.name}</span>
                {onRemoveAttachment ? (
                  <button
                    aria-label={`${text.removeAttachment}: ${file.name}`}
                    className="text-muted-foreground hover:text-destructive focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    onClick={() => onRemoveAttachment(index)}
                    type="button"
                  >
                    <X aria-hidden="true" className="size-3" />
                  </button>
                ) : null}
              </li>
            ))}
          </ul>
        ) : null}
        <div className="flex items-end gap-2">
          <label className="sr-only" htmlFor={`chat-draft-${roomId}`}>
            {text.writeMessage}
          </label>
          <Textarea
            className="min-h-10 max-h-32 flex-1 resize-none border-0 bg-transparent shadow-none focus-visible:ring-0"
            id={`chat-draft-${roomId}`}
            onChange={(event) => onDraftChange(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
                event.preventDefault();
                event.currentTarget.form?.requestSubmit();
              }
            }}
            placeholder={text.writeMessage}
            value={draft}
          />
          {attachable ? (
            <label
              className="inline-flex size-10 shrink-0 cursor-pointer items-center justify-center rounded-md text-muted-foreground hover:bg-muted focus-within:ring-2 focus-within:ring-ring"
              title={text.attachFiles}
            >
              <Paperclip aria-hidden="true" className="size-4" />
              <span className="sr-only">{text.attachFiles}</span>
              <input
                aria-label={text.attachFiles}
                className="sr-only"
                multiple
                onChange={(event) => {
                  const files = Array.from(event.target.files ?? []);
                  if (files.length > 0) {
                    onAttachFiles?.(files);
                  }
                  event.target.value = "";
                }}
                type="file"
              />
            </label>
          ) : null}
          <Button
            aria-label={text.send}
            className="size-10 shrink-0"
            disabled={sending || !draft.trim()}
            size="icon"
            type="submit"
          >
            <Send aria-hidden="true" className="size-4" />
          </Button>
        </div>
      </form>
    </section>
  );
}
