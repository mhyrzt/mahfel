"use client";

import { useCallback, useEffect, useState, type ChangeEvent, type ReactNode } from "react";
import Image from "next/image";

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
}: Readonly<{ context: FileContext; locale: Locale; canDelete: boolean; actorId?: string }>) {
  const text = roomCopy(locale);
  const { kind, id } = context;
  const [files, setFiles] = useState<FileRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [preview, setPreview] = useState<{ url: string; name: string; mime: string } | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setFiles(await listFiles({ kind, id }));
      setError(null);
    } catch {
      setError(text.fileUnavailable);
    } finally {
      setLoading(false);
    }
  }, [kind, id, text.fileUnavailable]);

  useEffect(() => {
    void Promise.resolve().then(refresh);
  }, [refresh]);

  useEffect(() => {
    return () => {
      if (preview) {
        URL.revokeObjectURL(preview.url);
      }
    };
  }, [preview]);

  async function onFileChange(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    setUploading(true);
    setError(null);
    try {
      await uploadFile(file, { kind, id });
      await refresh();
    } catch {
      setError(text.uploadFailed);
    } finally {
      setUploading(false);
      event.target.value = "";
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
    <section aria-label={text.files} className="space-y-4" data-testid={`${kind}-files`}>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-lg font-semibold">{text.files}</h2>
        <label className="inline-flex min-h-10 cursor-pointer items-center rounded-md border px-4 text-sm font-medium hover:bg-muted focus-within:ring-2 focus-within:ring-ring">
          {uploading ? text.uploading : text.upload}
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
          locale={locale}
          onDownload={(fileId) => void download(fileId)}
          onPreview={previewing ? undefined : (file) => void openPreview(file)}
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
