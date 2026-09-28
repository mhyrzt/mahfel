"use client";

import Image from "next/image";
import { useCallback, useEffect, useMemo, useState } from "react";
import * as Y from "yjs";

import { browserApiClient } from "@/lib/api/browser";
import {
  MahfelCollaborationProvider,
  type CollaborationEventFrame,
} from "@/lib/collaboration/provider";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import { PdfPage } from "./pdf-page";
import { PresentationViewer } from "./presentation-viewer";

interface PresentationState {
  file_ids: string[];
  current_page: number;
  total_pages: number;
}

export function PresentationPane({
  sessionId,
  canPresent,
  live,
  locale,
  onEvent,
}: Readonly<{
  sessionId: string;
  canPresent: boolean;
  live: boolean;
  locale: Locale;
  onEvent: (event: CollaborationEventFrame) => void;
}>) {
  const text = messagesFor(locale).collaboration;
  const [state, setState] = useState<PresentationState | null>(null);
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
      if (!data) {
        setMedia(null);
      }
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

  const change = useCallback(
    async (payload: Record<string, unknown>) => {
      try {
        await browserApiClient.post(
          `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/presentation`,
          payload,
        );
        await refresh();
      } catch {
        setFailed(true);
      }
    },
    [refresh, sessionId],
  );

  return (
    <PresentationViewer
      currentPage={state?.current_page ?? 1}
      totalPages={state?.total_pages ?? 0}
      canPresent={canPresent}
      live={live}
      locale={locale}
      failed={failed}
      onPageChange={(page) => void change({ action: "page", page })}
      onStop={() => void change({ action: "stop" })}
      renderPage={(page) => {
        if (page !== state?.current_page || !media || media.id !== activeId) {
          return (
            <span role="status" className="text-sm text-muted-foreground">
              {text.loading}
            </span>
          );
        }
        return media.mime === "application/pdf" ? (
          <PdfPage bytes={media.bytes} page={page} unavailable={text.pdfUnavailable} />
        ) : (
          <Image
            src={media.url}
            alt={text.presentation}
            width={1280}
            height={720}
            unoptimized
            className="mx-auto h-auto max-h-full max-w-full object-contain"
          />
        );
      }}
    />
  );
}
