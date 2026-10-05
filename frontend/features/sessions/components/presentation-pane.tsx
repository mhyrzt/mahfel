"use client";

import { Alert, AlertDescription } from "@/components/ui/alert";
import Image from "next/image";
import dynamic from "next/dynamic";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Square } from "lucide-react";
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
import { PresentationViewer } from "./presentation-viewer";
import type { PlaybackState } from "./presentation-playback";

const PresentationPlayer = dynamic(
  () => import("./presentation-player").then((module) => module.PresentationPlayer),
  { ssr: false },
);

interface PresentationPlayback {
  position_ms: number;
  paused: boolean;
  rate: number;
}

interface PresentationState {
  file_ids: string[];
  current_page: number;
  total_pages: number;
  playback?: PresentationPlayback | null;
}

function isVideoMime(mime: string): boolean {
  return mime.startsWith("video/");
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
        if (isVideoMime(data.file.mime)) {
          if (!cancelled) {
            setMedia({
              id: activeId,
              bytes: new Uint8Array(),
              mime: data.file.mime,
              url: data.url,
            });
          }
          return;
        }
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

  const emitPlayback = useCallback(
    (playback: PlaybackState) => {
      void browserApiClient
        .post(`/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/presentation`, {
          action: "playback",
          position_ms: playback.positionMs,
          paused: playback.paused,
          rate: playback.rate,
        })
        .catch(() => setFailed(true));
    },
    [sessionId],
  );

  if (media && isVideoMime(media.mime)) {
    const ready = media.id === activeId;
    const playback: PlaybackState | null = state?.playback
      ? {
          positionMs: state.playback.position_ms,
          paused: state.playback.paused,
          rate: state.playback.rate,
        }
      : null;
    return (
      <div className="flex h-full min-h-0 flex-1 flex-col gap-3 overflow-hidden rounded-xl border bg-background p-3">
        {failed ? (
          <Alert variant="destructive" className="text-sm text-destructive">
            <AlertDescription>{text.unavailable}</AlertDescription>
          </Alert>
        ) : null}
        <div className="min-h-0 flex-1">
          {ready ? (
            <PresentationPlayer
              canPresent={canPresent}
              locale={locale}
              onPlayback={emitPlayback}
              playback={playback}
              url={media.url}
            />
          ) : (
            <p role="status" className="p-6 text-sm text-muted-foreground">
              {text.loading}
            </p>
          )}
        </div>
        <div className="flex items-center justify-between gap-3 border-t pt-3">
          <span className="text-xs font-medium text-muted-foreground">{text.presentation}</span>
          {canPresent && live ? (
            <Button onClick={() => void change({ action: "stop" })} size="sm" variant="outline">
              <Square aria-hidden="true" className="size-3 fill-current" />
              {text.stop}
            </Button>
          ) : null}
        </div>
      </div>
    );
  }

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
            className="h-full w-full object-contain"
          />
        );
      }}
    />
  );
}
