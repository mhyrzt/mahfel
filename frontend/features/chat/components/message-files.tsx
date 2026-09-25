"use client";

import { useState, type ChangeEvent } from "react";
import { Paperclip } from "lucide-react";

import {
  Attachment,
  AttachmentContent,
  AttachmentDescription,
  AttachmentMedia,
  AttachmentTitle,
} from "@/components/ui/attachment";
import { Button } from "@/components/ui/button";
import { FileAttachment } from "@/features/files/components/file-attachment";
import { getFileDownloadUrl, listFiles, uploadFile } from "@/features/files/client";
import { roomCopy } from "@/features/rooms/room-copy";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

type FileRecord = components["schemas"]["FileResponse"];

export function MessageFiles({
  messageId,
  locale,
  canAttach,
}: Readonly<{ messageId: string; locale: Locale; canAttach: boolean }>) {
  const text = roomCopy(locale);
  const [open, setOpen] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [loading, setLoading] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [uploadName, setUploadName] = useState<string | null>(null);
  const [files, setFiles] = useState<FileRecord[]>([]);
  const [error, setError] = useState(false);

  async function refresh() {
    setLoading(true);
    try {
      setFiles(await listFiles({ kind: "chat_message", id: messageId }));
      setLoaded(true);
      setError(false);
    } catch {
      setError(true);
    } finally {
      setLoading(false);
    }
  }

  async function change(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    setUploading(true);
    setUploadName(file.name);
    try {
      await uploadFile(file, { kind: "chat_message", id: messageId });
      await refresh();
      setOpen(true);
    } catch {
      setError(true);
    } finally {
      event.target.value = "";
      setUploading(false);
      setUploadName(null);
    }
  }

  async function download(fileId: string) {
    try {
      window.location.assign(await getFileDownloadUrl(fileId));
    } catch {
      setError(true);
    }
  }

  return (
    <div data-testid={`message-files-${messageId}`}>
      <div className="flex items-center gap-1">
        <Button
          aria-expanded={open}
          onClick={() => {
            setOpen((previous) => !previous);
            if (!open) {
              void refresh();
            }
          }}
          size="sm"
          type="button"
          variant="ghost"
        >
          <Paperclip aria-hidden="true" className="size-3.5" />
          {text.attachments}
          {loaded && files.length > 0 ? ` ${files.length}` : ""}
        </Button>
        {canAttach ? (
          <label className="inline-flex h-8 cursor-pointer items-center rounded-md px-2 text-xs font-medium text-muted-foreground hover:bg-muted focus-within:ring-2 focus-within:ring-ring">
            {uploading ? text.uploading : text.attachFile}
            <input
              aria-label={text.attachFile}
              className="sr-only"
              disabled={uploading}
              onChange={(event) => void change(event)}
              type="file"
            />
          </label>
        ) : null}
      </div>
      {open ? (
        <div className="mt-2 space-y-2">
          {uploading && uploadName ? (
            <Attachment size="sm" state="uploading">
              <AttachmentMedia>
                <Paperclip aria-hidden="true" />
              </AttachmentMedia>
              <AttachmentContent>
                <AttachmentTitle>{uploadName}</AttachmentTitle>
                <AttachmentDescription>{text.uploading}</AttachmentDescription>
              </AttachmentContent>
            </Attachment>
          ) : null}
          {loading ? (
            <p role="status" className="text-xs text-muted-foreground">
              {text.loadingFiles}
            </p>
          ) : null}
          {error ? (
            <p role="alert" className="text-xs text-destructive">
              {text.fileUnavailable}
            </p>
          ) : null}
          {!loading && !error && files.length === 0 ? (
            <p className="text-xs text-muted-foreground">{text.noMessageFiles}</p>
          ) : null}
          {files.map((file) => (
            <FileAttachment
              compact
              file={file}
              key={file.id}
              locale={locale}
              onDownload={(id) => void download(id)}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}
