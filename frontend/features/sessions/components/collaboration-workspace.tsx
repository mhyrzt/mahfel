"use client";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { useCallback, useEffect, useState } from "react";
import dynamic from "next/dynamic";

import { NotesEditor } from "@/features/notes/components/notes-editor";
import { PresentationPane } from "./presentation-pane";

const WhiteboardEditor = dynamic(
  () =>
    import("@/features/whiteboard/components/whiteboard-editor").then(
      (module) => module.WhiteboardEditor,
    ),
  { ssr: false },
);
import { browserApiClient } from "@/lib/api/browser";
import type { CollaborationEventFrame } from "@/lib/collaboration/provider";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

interface Metadata {
  room_id: string;
  can_edit: boolean;
  can_present: boolean;
  can_manage: boolean;
  live: boolean;
  documents: { id: string; kind: string; schema_version: number }[];
}

export function CollaborationWorkspace({
  displayName,
  sessionId,
  locale,
  selectedSurface = "notes",
}: Readonly<{
  displayName?: string;
  sessionId: string;
  locale: Locale;
  selectedSurface?: "notes" | "whiteboard" | "presentation";
}>) {
  const text = messagesFor(locale).collaboration;
  const [metadata, setMetadata] = useState<Metadata | null>(null);
  const [failed, setFailed] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const { data } = await browserApiClient.get<Metadata>(
        `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration`,
      );
      setMetadata(data);
      setFailed(false);
    } catch {
      setFailed(true);
    }
  }, [sessionId]);

  useEffect(() => {
    void Promise.resolve().then(refresh);
  }, [refresh]);
  const onEvent = useCallback(
    (event: CollaborationEventFrame) => {
      if (event.kind === "edit_grant") {
        void refresh();
      }
    },
    [refresh],
  );

  let surface = null;
  if (metadata && selectedSurface === "notes") {
    surface = (
      <NotesEditor
        displayName={displayName}
        sessionId={sessionId}
        canEdit={metadata.can_edit}
        canManage={metadata.can_manage}
        live={metadata.live}
        locale={locale}
        onEvent={onEvent}
      />
    );
  } else if (metadata && selectedSurface === "whiteboard") {
    surface = (
      <WhiteboardEditor
        displayName={displayName}
        sessionId={sessionId}
        canEdit={metadata.can_edit}
        live={metadata.live}
        locale={locale}
        onEvent={onEvent}
      />
    );
  } else if (metadata) {
    surface = (
      <PresentationPane
        sessionId={sessionId}
        canPresent={metadata.can_present}
        live={metadata.live}
        locale={locale}
        onEvent={onEvent}
      />
    );
  }

  return (
    <section className="flex h-full min-h-0 flex-col gap-2" aria-label={text.title}>
      {failed ? (
        <Alert variant="destructive" className="text-sm text-destructive">
          <AlertDescription>{text.unavailable}</AlertDescription>
        </Alert>
      ) : null}
      {!metadata ? <p className="text-sm text-muted-foreground">{text.loading}</p> : surface}
    </section>
  );
}
