"use client";

import { useCallback, useEffect, useState } from "react";
import dynamic from "next/dynamic";

import { Button } from "@/components/ui/button";
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
  can_manage: boolean;
  live: boolean;
  documents: { id: string; kind: string; schema_version: number }[];
}

interface ParticipantGrant {
  actor_id: string;
  display_name: string;
  can_edit: boolean;
}

export function CollaborationWorkspace({
  sessionId,
  locale,
}: Readonly<{ sessionId: string; locale: Locale }>) {
  const text = messagesFor(locale).collaboration;
  const [metadata, setMetadata] = useState<Metadata | null>(null);
  const [failed, setFailed] = useState(false);
  const [tab, setTab] = useState<"notes" | "whiteboard" | "presentation">("notes");
  const [grants, setGrants] = useState<ParticipantGrant[]>([]);

  const refresh = useCallback(async () => {
    try {
      const { data } = await browserApiClient.get<Metadata>(
        `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration`,
      );
      setMetadata(data);
      setFailed(false);
      if (data.can_manage) {
        const response = await browserApiClient.get<{ participants: ParticipantGrant[] }>(
          `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/grants`,
        );
        setGrants(response.data.participants);
      }
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

  const changeGrant = async (participant: ParticipantGrant) => {
    try {
      await browserApiClient.put(
        `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/grants/${encodeURIComponent(participant.actor_id)}`,
        { can_edit: !participant.can_edit },
      );
      await refresh();
    } catch {
      setFailed(true);
    }
  };

  let surface = null;
  if (metadata && tab === "notes") {
    surface = (
      <NotesEditor
        sessionId={sessionId}
        canEdit={metadata.can_edit}
        canManage={metadata.can_manage}
        live={metadata.live}
        locale={locale}
        onEvent={onEvent}
      />
    );
  } else if (metadata && tab === "whiteboard") {
    surface = (
      <WhiteboardEditor
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
        roomId={metadata.room_id}
        canEdit={metadata.can_edit}
        live={metadata.live}
        locale={locale}
        onEvent={onEvent}
      />
    );
  }

  return (
    <section className="space-y-3" aria-label={text.title}>
      <div className="flex flex-wrap items-center gap-3">
        <h2 className="text-lg font-semibold">{text.title}</h2>
        {metadata ? (
          <span className="text-xs text-muted-foreground">
            {metadata.can_edit ? text.canEdit : text.viewOnly}
          </span>
        ) : null}
      </div>
      {failed ? (
        <p role="alert" className="text-sm text-destructive">
          {text.unavailable}
        </p>
      ) : null}
      {!metadata ? (
        <p className="text-sm text-muted-foreground">{text.loading}</p>
      ) : (
        <>
          <div className="flex gap-2" role="tablist" aria-label={text.title}>
            <Button
              role="tab"
              aria-selected={tab === "notes"}
              variant={tab === "notes" ? "default" : "outline"}
              onClick={() => setTab("notes")}
            >
              {text.notes}
            </Button>
            <Button
              role="tab"
              aria-selected={tab === "whiteboard"}
              variant={tab === "whiteboard" ? "default" : "outline"}
              onClick={() => setTab("whiteboard")}
            >
              {text.whiteboard}
            </Button>
            <Button
              role="tab"
              aria-selected={tab === "presentation"}
              variant={tab === "presentation" ? "default" : "outline"}
              onClick={() => setTab("presentation")}
            >
              {text.presentation}
            </Button>
          </div>
          {surface}
          {metadata.can_manage && metadata.live ? (
            <div className="space-y-2 rounded-lg border p-3">
              <h3 className="font-medium">{text.permissions}</h3>
              {grants.map((participant) => (
                <div key={participant.actor_id} className="flex items-center justify-between gap-3">
                  <span className="text-sm">{participant.display_name}</span>
                  <Button size="sm" variant="outline" onClick={() => void changeGrant(participant)}>
                    {participant.can_edit ? text.revokeEdit : text.allowEdit}
                  </Button>
                </div>
              ))}
            </div>
          ) : null}
        </>
      )}
    </section>
  );
}
