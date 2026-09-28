"use client";

import {
  useCallback,
  useEffect,
  useState,
  type ChangeEvent,
  type DragEvent,
  type ReactNode,
} from "react";
import Image from "next/image";
import { UploadCloud } from "lucide-react";

import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { roomCopy } from "@/features/rooms/room-copy";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { getFileDownloadUrl, listFiles, uploadFile, type FileContext } from "../client";
import { FileList } from "./file-list";

type FileRecord = components["schemas"]["FileResponse"];

export function ContextFileLibrary({
  context,
  locale,
  canDelete,
  actorId,
  compact = false,
  presentationSessionId,
  roomContextId,
  onPresented,
}: Readonly<{
  context: FileContext;
  locale: Locale;
  canDelete: boolean;
  actorId?: string;
  compact?: boolean;
  presentationSessionId?: string;
  roomContextId?: string;
  onPresented?: () => void;
}>) {
  const text = roomCopy(locale);
  const { kind, id } = context;
  const [files, setFiles] = useState<FileRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [preview, setPreview] = useState<{ url: string; name: string; mime: string } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [canPresent, setCanPresent] = useState(false);
  const [dragging, setDragging] = useState(false);
  const dropLabel =
    locale === "fa"
      ? "فایل را بکشید یا برای بارگذاری انتخاب کنید"
      : "Drop a file or click to upload";

  const refresh = useCallback(async () => {
    const [contextResult, roomResult] = await Promise.allSettled([
      listFiles({ kind, id }),
      roomContextId ? listFiles({ kind: "room", id: roomContextId }) : Promise.resolve([]),
    ]);
    setFiles([
      ...(contextResult.status === "fulfilled" ? contextResult.value : []),
      ...(roomResult.status === "fulfilled" ? roomResult.value : []),
    ]);
    setError(
      contextResult.status === "rejected" || roomResult.status === "rejected"
        ? text.fileUnavailable
        : null,
    );
    setLoading(false);
  }, [kind, id, roomContextId, text.fileUnavailable]);

  useEffect(() => {
    void Promise.resolve().then(refresh);
  }, [refresh]);

  useEffect(() => {
    if (!presentationSessionId) {
      return undefined;
    }
    let active = true;
    const refreshAccess = async () => {
      const response = await browserApiClient
        .get<{ can_present: boolean }>(
          `/v1/sessions/${encodeURIComponent(presentationSessionId)}/collaboration`,
        )
        .catch(() => null);
      if (active && response) {
        setCanPresent(response.data.can_present);
      }
    };
    void refreshAccess();
    const timer = window.setInterval(() => void refreshAccess(), 3_000);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, [presentationSessionId]);

  useEffect(() => {
    return () => {
      if (preview) {
        URL.revokeObjectURL(preview.url);
      }
    };
  }, [preview]);

  async function onFileChange(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (file) {
      await saveFile(file);
    }
    event.target.value = "";
  }

  async function saveFile(file: File) {
    setUploading(true);
    setError(null);
    try {
      await uploadFile(file, { kind, id });
      await refresh();
    } catch {
      setError(text.uploadFailed);
    } finally {
      setUploading(false);
    }
  }

  async function onDrop(event: DragEvent<HTMLElement>) {
    event.preventDefault();
    setDragging(false);
    const file = event.dataTransfer.files[0];
    if (file) {
      await saveFile(file);
    }
  }

  async function present(fileId: string) {
    if (!presentationSessionId || !canPresent) {
      return;
    }
    try {
      await browserApiClient.post(
        `/v1/sessions/${encodeURIComponent(presentationSessionId)}/collaboration/presentation`,
        { action: "select", file_ids: [fileId] },
      );
      setError(null);
      onPresented?.();
    } catch {
      setError(text.fileUnavailable);
    }
  }

  async function download(fileId: string) {
    try {
      window.location.assign(await getFileDownloadUrl(fileId));
    } catch {
      setError(text.downloadFailed);
    }
  }

  async function openPreview(file: FileRecord) {
    setPreviewing(true);
    setError(null);
    try {
      const signedUrl = new URL(await getFileDownloadUrl(file.id), window.location.origin);
      if (signedUrl.protocol !== "https:" && signedUrl.protocol !== "http:") {
        throw new Error("Unsupported file URL");
      }
      const response = await fetch(signedUrl, { credentials: "omit", redirect: "error" });
      if (!response.ok) {
        throw new Error("File preview transfer failed");
      }
      const blob = new Blob([await response.blob()], { type: file.mime });
      setPreview({ url: URL.createObjectURL(blob), name: file.file_name, mime: file.mime });
    } catch {
      setError(text.previewFailed);
    } finally {
      setPreviewing(false);
    }
  }

  async function remove(fileId: string) {
    try {
      await browserApiClient.delete(`/v1/files/${encodeURIComponent(fileId)}`);
      await refresh();
    } catch {
      setError(text.fileUnavailable);
    }
  }

  let previewContent: ReactNode = null;
  if (preview?.mime.startsWith("image/")) {
    previewContent = (
      <Image
        alt={preview.name}
        className="mx-auto h-auto max-h-[70dvh] w-auto max-w-full object-contain"
        height={800}
        src={preview.url}
        unoptimized
        width={1200}
      />
    );
  } else if (preview) {
    previewContent = (
      <iframe
        className="h-[70dvh] w-full rounded-md border"
        referrerPolicy="no-referrer"
        sandbox="allow-scripts"
        src={preview.url}
        title={preview.name}
      />
    );
  }

  return (
    <section aria-label={text.files} className="min-w-0 space-y-3" data-testid={`${kind}-files`}>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className={compact ? "text-base font-semibold" : "text-lg font-semibold"}>
          {text.files}
        </h2>
      </div>
      <div
        className={`rounded-lg border border-dashed transition-colors ${dragging ? "border-primary bg-primary/10" : "border-border/70 bg-muted/25"}`}
        onDragEnter={(event) => {
          event.preventDefault();
          setDragging(true);
        }}
        onDragOver={(event) => event.preventDefault()}
        onDragLeave={(event) => {
          event.preventDefault();
          setDragging(false);
        }}
        onDrop={(event) => void onDrop(event)}
      >
        <label className="flex min-h-20 cursor-pointer flex-col items-center justify-center gap-1 px-3 py-3 text-center text-xs text-muted-foreground focus-within:ring-2 focus-within:ring-ring">
          <UploadCloud aria-hidden="true" className="size-5" />
          <span>{uploading ? text.uploading : dropLabel}</span>
          <input
            aria-label={text.upload}
            className="sr-only"
            disabled={uploading}
            onChange={(event) => void onFileChange(event)}
            type="file"
          />
        </label>
      </div>
      {error ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : null}
      {loading ? null : (
        <FileList
          actorId={actorId}
          canDelete={canDelete}
          files={files}
          compact={compact}
          locale={locale}
          onDownload={(fileId) => void download(fileId)}
          onPreview={previewing ? undefined : (file) => void openPreview(file)}
          onPresent={canPresent ? (fileId) => void present(fileId) : undefined}
          onRemove={(fileId) => void remove(fileId)}
        />
      )}
      <Dialog onOpenChange={(open) => !open && setPreview(null)} open={preview !== null}>
        <DialogContent className="max-h-[calc(100dvh-2rem)] max-w-4xl overflow-y-auto">
          <DialogTitle>{preview?.name}</DialogTitle>
          <DialogDescription className="sr-only">{text.preview}</DialogDescription>
          {previewContent}
        </DialogContent>
      </Dialog>
    </section>
  );
}
