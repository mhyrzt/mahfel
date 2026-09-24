"use client";

import { ListItemNode, ListNode } from "@lexical/list";
import { LinkNode } from "@lexical/link";
import { CollaborationPlugin } from "@lexical/react/LexicalCollaborationPlugin";
import { LexicalCollaboration } from "@lexical/react/LexicalCollaborationContext";
import { LexicalComposer } from "@lexical/react/LexicalComposer";
import { ContentEditable } from "@lexical/react/LexicalContentEditable";
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext";
import { LexicalErrorBoundary } from "@lexical/react/LexicalErrorBoundary";
import { ListPlugin } from "@lexical/react/LexicalListPlugin";
import { RichTextPlugin } from "@lexical/react/LexicalRichTextPlugin";
import { HeadingNode } from "@lexical/rich-text";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as Y from "yjs";

import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import {
  MahfelCollaborationProvider,
  type CollaborationEventFrame,
} from "@/lib/collaboration/provider";

import { NotesToolbar } from "./notes-toolbar";

interface NotesEditorProps {
  sessionId: string;
  canEdit: boolean;
  canManage: boolean;
  live: boolean;
  locale: Locale;
  onEvent: (event: CollaborationEventFrame) => void;
}

function receiveNotesEvent(
  event: CollaborationEventFrame,
  scrollRef: React.RefObject<HTMLDivElement | null>,
  remoteScroll: React.RefObject<boolean>,
  onEvent: (event: CollaborationEventFrame) => void,
) {
  if (event.kind === "viewport" && event.data.surface === "notes") {
    const y = event.data.y;
    if (typeof y === "number" && scrollRef.current) {
      remoteScroll.current = true;
      scrollRef.current.scrollTop = y;
      window.setTimeout(() => {
        remoteScroll.current = false;
      }, 150);
    }
  }
  onEvent(event);
}

function EditableState({ canEdit }: Readonly<{ canEdit: boolean }>) {
  const [editor] = useLexicalComposerContext();
  useEffect(() => editor.setEditable(canEdit), [editor, canEdit]);
  return null;
}

export function NotesEditor({
  sessionId,
  canEdit: initialCanEdit,
  canManage,
  live,
  locale,
  onEvent,
}: Readonly<NotesEditorProps>) {
  const text = messagesFor(locale).collaboration;
  const [canEdit, setCanEdit] = useState(initialCanEdit);
  const [status, setStatus] = useState("connecting");
  const scrollRef = useRef<HTMLDivElement>(null);
  const remoteScroll = useRef(false);
  const lastScroll = useRef(0);
  const doc = useMemo(() => new Y.Doc({ guid: sessionId }), [sessionId]);
  const provider = useMemo(
    () =>
      new MahfelCollaborationProvider(
        sessionId,
        "notes",
        doc,
        setCanEdit,
        // eslint-disable-next-line react-hooks/refs
        (event) => receiveNotesEvent(event, scrollRef, remoteScroll, onEvent),
      ),
    [sessionId, doc, onEvent],
  );
  const providerFactory = useCallback(
    // eslint-disable-next-line react-hooks/preserve-manual-memoization
    (id: string, map: Map<string, Y.Doc>) => {
      map.set(id, doc);
      return provider;
    },
    [doc, provider],
  );

  useEffect(() => {
    const onStatus = ({ status }: { status: string }) => setStatus(status);
    provider.on("status", onStatus);
    return () => {
      provider.off("status", onStatus);
      provider.destroy();
      doc.destroy();
    };
  }, [doc, provider]);

  const onScroll = () => {
    if (!live || remoteScroll.current || !scrollRef.current) {
      return;
    }
    const now = Date.now();
    if (now - lastScroll.current < 150) {
      return;
    }
    lastScroll.current = now;
    void browserApiClient.post(
      `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/viewport`,
      {
        surface: "notes",
        x: 0,
        y: scrollRef.current.scrollTop,
      },
    );
  };

  return (
    <div className="overflow-hidden rounded-lg border bg-background">
      <p className="border-b px-3 py-2 text-xs text-muted-foreground" role="status">
        {status === "connected" ? text.connected : text.loading}
      </p>
      <LexicalCollaboration>
        <LexicalComposer
          initialConfig={{
            namespace: `mahfel-notes-${sessionId}`,
            editable: canEdit,
            editorState: null,
            nodes: [HeadingNode, ListNode, ListItemNode, LinkNode],
            onError: (error) => {
              throw error;
            },
            theme: { paragraph: "mb-2", heading: { h2: "text-xl font-semibold" } },
          }}
        >
          <EditableState canEdit={canEdit} />
          <NotesToolbar disabled={!canEdit} locale={locale} />
          <div ref={scrollRef} onScroll={onScroll} className="h-[28rem] overflow-y-auto p-4">
            <RichTextPlugin
              contentEditable={
                <ContentEditable
                  aria-label={text.notesEditor}
                  className="min-h-full outline-none"
                />
              }
              placeholder={
                <span className="pointer-events-none text-muted-foreground">
                  {text.notesPlaceholder}
                </span>
              }
              ErrorBoundary={LexicalErrorBoundary}
            />
            <ListPlugin />
            <CollaborationPlugin
              id={`notes-${sessionId}`}
              providerFactory={providerFactory}
              shouldBootstrap={canManage}
            />
          </div>
        </LexicalComposer>
      </LexicalCollaboration>
    </div>
  );
}
