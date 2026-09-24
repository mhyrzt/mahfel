"use client";

import Image from "next/image";
import { useCallback, useEffect, useMemo, useState } from "react";
import * as Y from "yjs";

import { Button } from "@/components/ui/button";
import { browserApiClient } from "@/lib/api/browser";
import {
  MahfelCollaborationProvider,
  type CollaborationEventFrame,
} from "@/lib/collaboration/provider";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import { PdfPage } from "./pdf-page";

interface PresentationState {
  file_ids: string[];
  current_page: number;
  total_pages: number;
}

interface LibraryFile {
  id: string;
  file_name: string;
  mime: string;
}

export function PresentationPane({
  sessionId,
  roomId,
  canEdit,
  live,
  locale,
  onEvent,
}: Readonly<{
  sessionId: string;
  roomId: string;
  canEdit: boolean;
  live: boolean;
  locale: Locale;
  onEvent: (event: CollaborationEventFrame) => void;
}>) {
  const text = messagesFor(locale).collaboration;
  const [state, setState] = useState<PresentationState | null>(null);
  const [files, setFiles] = useState<LibraryFile[]>([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [media, setMedia] = useState<{
    id: string;
    bytes: Uint8Array;
    mime: string;
    url: string;
  } | null>(null);
  const [failed, setFailed] = useState(false);
  const doc = useMemo(() => new Y.Doc({ guid: sessionId }), [sessionId]);
  const refresh = useCallback(async () => {
    try {
      const { data } = await browserApiClient.get<PresentationState | null>(
        `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/presentation`,
      );
      setState(data);
      setFailed(false);
    } catch {
      setFailed(true);
    }
  }, [sessionId]);
  const provider = useMemo(
    () =>
      new MahfelCollaborationProvider(
        sessionId,
        "notes",
        doc,
        () => {},
        (event) => {
          if (event.kind === "presentation") {
            void refresh();
          }
          onEvent(event);
        },
      ),
    [doc, onEvent, refresh, sessionId],
  );

  useEffect(() => {
    void Promise.resolve().then(refresh);
    provider.connect();
    return () => {
      provider.destroy();
      doc.destroy();
    };
  }, [doc, provider, refresh]);

  useEffect(() => {
    if (!canEdit || !live) {
      return;
    }
    void Promise.all([
      browserApiClient.get<{ files: LibraryFile[] }>(
        `/v1/files?context_kind=room&context_id=${encodeURIComponent(roomId)}`,
      ),
      browserApiClient.get<{ files: LibraryFile[] }>(
        `/v1/files?context_kind=session&context_id=${encodeURIComponent(sessionId)}`,
      ),
    ])
      .then(([room, session]) => {
        setFiles(
          [...room.data.files, ...session.data.files].filter((file) =>
            ["application/pdf", "image/png", "image/jpeg"].includes(file.mime),
          ),
        );
      })
      .catch(() => setFailed(true));
  }, [canEdit, live, roomId, sessionId]);

  const activeId =
    state?.file_ids.length === 1
      ? state.file_ids[0]
      : state?.file_ids[(state.current_page ?? 1) - 1];
  useEffect(() => {
    if (!activeId) {
      return () => {};
    }
    let cancelled = false;
    void (async () => {
      try {
        const { data } = await browserApiClient.get<{
          url: string;
          headers: { name: string; value: string }[];
          file: { mime: string };
        }>(`/v1/files/${encodeURIComponent(activeId)}/download`);
        const response = await fetch(data.url, {
          headers: Object.fromEntries(data.headers.map((header) => [header.name, header.value])),
        });
        if (!response.ok) {
          throw new Error("file unavailable");
        }
        const bytes = new Uint8Array(await response.arrayBuffer());
        if (!cancelled) {
          const url = URL.createObjectURL(new Blob([bytes], { type: data.file.mime }));
          setMedia({ id: activeId, bytes, mime: data.file.mime, url });
        }
      } catch {
        if (!cancelled) {
          setFailed(true);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [activeId]);

  useEffect(
    () => () => {
      if (media) {
        URL.revokeObjectURL(media.url);
      }
    },
    [media],
  );

  const choose = (file: LibraryFile) => {
    setSelected((current) => {
      if (file.mime === "application/pdf") {
        return [file.id];
      }
      if (current.includes(file.id)) {
        return current.filter((id) => id !== file.id);
      }
      return [
        ...current.filter((id) => files.find((item) => item.id === id)?.mime !== "application/pdf"),
        file.id,
      ];
    });
  };

  const change = async (payload: Record<string, unknown>) => {
    try {
      await browserApiClient.post(
        `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/presentation`,
        payload,
      );
      await refresh();
    } catch {
      setFailed(true);
    }
  };

  return (
    <div className="space-y-3 rounded-lg border p-3">
      {failed ? (
        <p role="alert" className="text-sm text-destructive">
          {text.unavailable}
        </p>
      ) : null}
      {canEdit && live ? (
        <div className="space-y-2">
          <p className="text-sm font-medium">{text.selectFiles}</p>
          <div className="flex flex-wrap gap-2">
            {files.map((file) => (
              <Button
                key={file.id}
                size="sm"
                variant={selected.includes(file.id) ? "default" : "outline"}
                onClick={() => choose(file)}
                aria-pressed={selected.includes(file.id)}
              >
                {selected.includes(file.id) ? `${selected.indexOf(file.id) + 1}. ` : ""}
                {file.file_name}
              </Button>
            ))}
          </div>
          <div className="flex gap-2">
            <Button
              disabled={!selected.length}
              onClick={() => void change({ action: "select", file_ids: selected })}
            >
              {text.present}
            </Button>
            <Button
              variant="outline"
              disabled={!state}
              onClick={() => void change({ action: "stop" })}
            >
              {text.stop}
            </Button>
          </div>
        </div>
      ) : null}
      {state && media && media.id === activeId ? (
        <div className="space-y-2">
          <div className="max-h-[38rem] overflow-auto bg-muted p-2">
            {media.mime === "application/pdf" ? (
              <PdfPage
                bytes={media.bytes}
                page={state.current_page}
                unavailable={text.pdfUnavailable}
              />
            ) : (
              <Image
                src={media.url}
                alt={text.presentation}
                width={1280}
                height={720}
                unoptimized
                className="mx-auto h-auto max-w-full"
              />
            )}
          </div>
          <div className="flex items-center justify-center gap-3">
            <Button
              variant="outline"
              disabled={!canEdit || !live || state.current_page <= 1}
              onClick={() => void change({ action: "page", page: state.current_page - 1 })}
            >
              {text.previous}
            </Button>
            <span aria-live="polite">
              {state.current_page} / {state.total_pages}
            </span>
            <Button
              variant="outline"
              disabled={!canEdit || !live || state.current_page >= state.total_pages}
              onClick={() => void change({ action: "page", page: state.current_page + 1 })}
            >
              {text.next}
            </Button>
          </div>
        </div>
      ) : (
        <p className="text-sm text-muted-foreground">{text.noPresentation}</p>
      )}
    </div>
  );
}
