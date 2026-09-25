"use client";

import { useCallback, useEffect, useState } from "react";
import { ChevronDown, Files } from "lucide-react";

import { FileAttachment } from "@/features/files/components/file-attachment";
import { getFileDownloadUrl, listFiles } from "@/features/files/client";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";
import { cn } from "@/lib/utils";

import { roomCopy } from "./room-copy";
import { SessionFilePicker } from "./session-file-picker";

type FileRecord = components["schemas"]["FileResponse"];

export function SessionFiles({
  sessionId,
  roomId,
  locale,
  initialFiles,
  initialUsedIds,
  initialRoomFiles,
  attachRoomFile,
  uploadSessionFile,
}: Readonly<{
  sessionId: string;
  roomId: string;
  locale: Locale;
  initialFiles?: FileRecord[];
  initialUsedIds?: string[];
  initialRoomFiles?: FileRecord[];
  attachRoomFile?: (file: FileRecord) => Promise<FileRecord>;
  uploadSessionFile?: (file: File) => Promise<FileRecord>;
}>) {
  const text = roomCopy(locale);
  const [open, setOpen] = useState(true);
  const [loading, setLoading] = useState(initialFiles === undefined);
  const [files, setFiles] = useState<FileRecord[]>(initialFiles ?? []);
  const [usedIds, setUsedIds] = useState<string[]>(initialUsedIds ?? []);
  const [error, setError] = useState(false);
  const [loaded, setLoaded] = useState(initialFiles !== undefined);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const listed = await listFiles({ kind: "session", id: sessionId }, 100);
      const presentation = await browserApiClient
        .get<{ file_ids: string[] } | null>(
          `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/presentation`,
        )
        .catch(() => null);
      const selected = presentation?.data?.file_ids ?? [];
      const fromRoom = selected.some((id) => !listed.some((file) => file.id === id))
        ? (await listFiles({ kind: "room", id: roomId }, 100)).filter((file) =>
            selected.includes(file.id),
          )
        : [];
      setFiles([...listed, ...fromRoom]);
      setUsedIds(selected);
      setLoaded(true);
      setError(false);
    } catch {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, [roomId, sessionId]);

  useEffect(() => {
    if (initialFiles === undefined) {
      void Promise.resolve().then(load);
    }
  }, [initialFiles, load]);

  function toggle() {
    setOpen((value) => !value);
    if (!open && error) {
      void load();
    }
  }

  async function download(fileId: string) {
    try {
      window.location.assign(await getFileDownloadUrl(fileId));
    } catch {
      setError(true);
    }
  }

  const used = files.filter((file) => usedIds.includes(file.id));
  const attached = files.filter((file) => !usedIds.includes(file.id));
  const panelId = `session-files-${sessionId}`;
  return (
    <div className="border-t border-border/70">
      <button
        aria-controls={panelId}
        aria-expanded={open}
        className="flex min-h-11 w-full items-center gap-2 px-4 text-start text-sm font-medium text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring sm:px-6"
        data-testid={`session-files-toggle-${sessionId}`}
        onClick={toggle}
        type="button"
      >
        <Files aria-hidden="true" className="size-4" />
        <span>{text.sessionFiles}</span>
        {loaded ? <span className="text-xs tabular-nums">{files.length}</span> : null}
        <ChevronDown
          aria-hidden="true"
          className={cn("ms-auto size-4 transition-transform", open && "rotate-180")}
        />
      </button>
      {open ? (
        <div
          className="space-y-5 border-t border-border/60 bg-muted/20 px-4 py-5 sm:px-6"
          id={panelId}
        >
          {loading ? (
            <p className="text-sm text-muted-foreground" role="status">
              {text.loadingFiles}
            </p>
          ) : null}
          {error ? (
            <p className="text-sm text-destructive" role="alert">
              {text.fileUnavailable}
            </p>
          ) : null}
          {!loading && !error && files.length === 0 ? (
            <p className="text-sm text-muted-foreground">{text.noSessionFiles}</p>
          ) : null}
          <div
            className="max-h-72 space-y-5 overflow-y-auto overscroll-contain pe-1"
            data-testid={`session-attachments-${sessionId}`}
          >
            {used.length > 0 ? (
              <div className="space-y-2">
                <h4 className="text-xs font-semibold tracking-wide text-muted-foreground">
                  {text.usedFiles}
                </h4>
                <ul className="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
                  {used.map((file) => (
                    <li key={file.id}>
                      <FileAttachment
                        compact
                        file={file}
                        locale={locale}
                        onDownload={(id) => void download(id)}
                      />
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
            {attached.length > 0 ? (
              <div className="space-y-2">
                <h4 className="text-xs font-semibold tracking-wide text-muted-foreground">
                  {text.attachedFiles}
                </h4>
                <ul className="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
                  {attached.map((file) => (
                    <li key={file.id}>
                      <FileAttachment
                        compact
                        file={file}
                        locale={locale}
                        onDownload={(id) => void download(id)}
                      />
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
          </div>
          <SessionFilePicker
            sessionId={sessionId}
            roomId={roomId}
            locale={locale}
            attachedIds={files.map((file) => file.id)}
            initialRoomFiles={initialRoomFiles}
            attachRoomFile={attachRoomFile}
            uploadSessionFile={uploadSessionFile}
            onAdded={(file) => {
              setFiles((current) =>
                current.some((item) => item.id === file.id) ? current : [file, ...current],
              );
              setLoaded(true);
            }}
          />
        </div>
      ) : null}
    </div>
  );
}
