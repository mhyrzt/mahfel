"use client";

import { useCallback, useEffect, useState } from "react";
import { Film } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import type { RecordingExport, RecordingListItem } from "./export-types";
import { RecordingListView } from "./recording-list-view";
import { RecordingPlayer } from "./recording-player";

export function RecordingList({
  sessionId,
  locale,
  canManage = false,
}: Readonly<{ sessionId: string; locale: Locale; canManage?: boolean }>) {
  const [open, setOpen] = useState(false);
  const [items, setItems] = useState<RecordingListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [failed, setFailed] = useState(false);
  const [actionFailed, setActionFailed] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [playingId, setPlayingId] = useState<string | null>(null);
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const path = `/v1/sessions/${encodeURIComponent(sessionId)}/recordings`;
  const refresh = useCallback(
    async (signal?: AbortSignal) => {
      try {
        const response = await browserApiClient.get<Omit<RecordingListItem, "export">[]>(path, {
          signal,
        });
        const values: RecordingListItem[] = [];
        for (let offset = 0; offset < response.data.length; offset += 4) {
          const batch = await Promise.all(
            response.data.slice(offset, offset + 4).map(async (recording) => {
              let exported: RecordingExport | null = null;
              if (recording.status === "ready") {
                try {
                  exported = (
                    await browserApiClient.get<RecordingExport>(`${path}/${recording.id}/export`, {
                      signal,
                    })
                  ).data;
                } catch {
                  exported = null;
                }
              }
              return { ...recording, export: exported };
            }),
          );
          values.push(...batch);
        }
        if (!signal?.aborted) {
          setItems(values);
          setFailed(false);
          setLoading(false);
        }
      } catch {
        if (!signal?.aborted) {
          setFailed(true);
          setLoading(false);
        }
      }
    },
    [path],
  );
  useEffect(() => {
    if (!open) {
      return undefined;
    }
    const abort = new AbortController();
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      await refresh(abort.signal);
      if (!abort.signal.aborted) {
        timer = setTimeout(() => void poll(), 5000);
      }
    };
    void poll();
    return () => {
      abort.abort();
      clearTimeout(timer);
    };
  }, [open, refresh]);
  const action = async (id: string, download: boolean) => {
    setBusyId(id);
    setActionFailed(false);
    try {
      if (download) {
        const { data } = await browserApiClient.get<{ url: string }>(
          `${path}/${encodeURIComponent(id)}/export/download`,
        );
        const url = new URL(data.url);
        if (url.protocol !== "https:" && url.protocol !== "http:") {
          throw new Error("Invalid recording download URL");
        }
        const link = document.createElement("a");
        link.href = url.href;
        link.download = `recording-${id}.mp4`;
        link.rel = "noopener noreferrer";
        link.click();
      } else {
        await browserApiClient.post(`${path}/${encodeURIComponent(id)}/export`);
        await refresh();
      }
    } catch {
      setActionFailed(true);
    } finally {
      setBusyId(null);
    }
  };
  const remove = async (id: string) => {
    setBusyId(id);
    setActionFailed(false);
    try {
      await browserApiClient.delete(`${path}/${encodeURIComponent(id)}`);
      setConfirmId(null);
      setItems((items) => items.filter((item) => item.id !== id));
      await refresh();
    } catch {
      setActionFailed(true);
    } finally {
      setBusyId(null);
    }
  };
  const retryPlayback = async (id: string) => {
    setBusyId(id);
    setActionFailed(false);
    try {
      await browserApiClient.post(`${path}/${encodeURIComponent(id)}/playback/retry`);
      await refresh();
    } catch {
      setActionFailed(true);
    } finally {
      setBusyId(null);
    }
  };
  const text = messagesFor(locale).recordingExports;
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button aria-label={text.title} size="icon" variant="outline">
          <Film aria-hidden="true" className="size-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{text.title}</DialogTitle>
        </DialogHeader>
        <RecordingListView
          items={items}
          locale={locale}
          canManage={canManage}
          loading={loading}
          failed={failed}
          actionFailed={actionFailed}
          busyId={busyId}
          onDownload={(id) => void action(id, true)}
          onRetry={(id) => void action(id, false)}
          onRetryPlayback={(id) => void retryPlayback(id)}
          onPlay={setPlayingId}
          onRemove={setConfirmId}
        />
      </DialogContent>
      <Dialog open={playingId !== null} onOpenChange={(next) => !next && setPlayingId(null)}>
        <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-3xl">
          <DialogHeader>
            <DialogTitle>{messagesFor(locale).recordingPlayback.title}</DialogTitle>
          </DialogHeader>
          {playingId ? (
            <RecordingPlayer sessionId={sessionId} recordingId={playingId} locale={locale} />
          ) : null}
        </DialogContent>
      </Dialog>
      <Dialog
        open={confirmId !== null}
        onOpenChange={(next) => !next && busyId === null && setConfirmId(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{text.removeTitle}</DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground">{text.removeDescription}</p>
          <div className="flex justify-end gap-2">
            <Button variant="outline" disabled={busyId !== null} onClick={() => setConfirmId(null)}>
              {text.removeCancel}
            </Button>
            <Button
              variant="destructive"
              disabled={busyId !== null}
              onClick={() => confirmId && void remove(confirmId)}
            >
              {text.remove}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </Dialog>
  );
}
