"use client";

import { useEffect, useState } from "react";
import Image from "next/image";
import { Download, Eye, File, FileImage, FileText, Presentation, Trash2 } from "lucide-react";

import {
  Attachment,
  AttachmentAction,
  AttachmentActions,
  AttachmentContent,
  AttachmentDescription,
  AttachmentMedia,
  AttachmentTitle,
  AttachmentTrigger,
} from "@/components/ui/attachment";
import { roomCopy } from "@/features/rooms/room-copy";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";
import { getFileDownloadUrl } from "../client";

type FileRecord = components["schemas"]["FileResponse"];

export function canPreviewFile(mime: string): boolean {
  return [
    "application/pdf",
    "text/plain",
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/avif",
  ].includes(mime);
}

function fileDescription(file: FileRecord, locale: Locale) {
  const extension = file.file_name.split(".").at(-1)?.toUpperCase() || file.mime;
  const size = new Intl.NumberFormat(locale === "fa" ? "fa-IR" : "en-US", {
    maximumFractionDigits: 1,
  }).format(file.size_bytes / (1024 * 1024));
  return `${extension} · ${size} MB`;
}

export function FileAttachment({
  file,
  locale,
  onPreview,
  onDownload,
  onRemove,
  onPresent,
  compact = false,
}: Readonly<{
  file: FileRecord;
  locale: Locale;
  onPreview?: (file: FileRecord) => void;
  onDownload?: (fileId: string) => void;
  onRemove?: (fileId: string) => void;
  onPresent?: (fileId: string) => void;
  compact?: boolean;
}>) {
  const text = roomCopy(locale);
  const previewable = canPreviewFile(file.mime) && Boolean(onPreview);
  const [thumbnail, setThumbnail] = useState<string | null>(null);
  useEffect(() => {
    if (!file.mime.startsWith("image/")) {
      return undefined;
    }
    let cancelled = false;
    let objectUrl: string | null = null;
    void (async () => {
      try {
        const signedUrl = new URL(await getFileDownloadUrl(file.id), window.location.origin);
        if (!["http:", "https:"].includes(signedUrl.protocol)) {
          return;
        }
        const response = await fetch(signedUrl, { credentials: "omit", redirect: "error" });
        if (!response.ok) {
          return;
        }
        objectUrl = URL.createObjectURL(await response.blob());
        if (!cancelled) {
          setThumbnail(objectUrl);
        } else {
          URL.revokeObjectURL(objectUrl);
        }
      } catch {}
    })();
    return () => {
      cancelled = true;
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    };
  }, [file.id, file.mime]);
  let media = <File aria-hidden="true" />;
  if (file.mime.startsWith("image/")) {
    media = thumbnail ? (
      <Image
        src={thumbnail}
        alt=""
        width={56}
        height={56}
        unoptimized
        className="size-full rounded-md object-cover"
      />
    ) : (
      <FileImage aria-hidden="true" />
    );
  } else if (file.mime === "application/pdf" || file.mime.startsWith("text/")) {
    media = <FileText aria-hidden="true" />;
  }
  return (
    <Attachment
      className="w-full border-border/70 bg-background/60 hover:bg-muted/40"
      data-testid={`attachment-${file.id}`}
      size={compact ? "sm" : "default"}
    >
      <AttachmentMedia
        className={file.mime.startsWith("image/") ? "size-11 overflow-hidden" : undefined}
      >
        {media}
      </AttachmentMedia>
      <AttachmentContent>
        <AttachmentTitle title={file.file_name}>{file.file_name}</AttachmentTitle>
        <AttachmentDescription>{fileDescription(file, locale)}</AttachmentDescription>
      </AttachmentContent>
      {previewable ? (
        <AttachmentTrigger
          aria-label={`${text.openAttachment}: ${file.file_name}`}
          onClick={() => onPreview?.(file)}
        />
      ) : null}
      <AttachmentActions>
        {onPresent ? (
          <AttachmentAction
            aria-label={`${locale === "fa" ? "ارائه" : "Present"}: ${file.file_name}`}
            onClick={() => onPresent(file.id)}
          >
            <Presentation aria-hidden="true" />
          </AttachmentAction>
        ) : null}
        {previewable ? (
          <AttachmentAction
            aria-label={`${text.preview}: ${file.file_name}`}
            onClick={() => onPreview?.(file)}
          >
            <Eye aria-hidden="true" />
          </AttachmentAction>
        ) : null}
        {onDownload ? (
          <AttachmentAction
            aria-label={`${text.download}: ${file.file_name}`}
            onClick={() => onDownload(file.id)}
          >
            <Download aria-hidden="true" />
          </AttachmentAction>
        ) : null}
        {onRemove ? (
          <AttachmentAction
            aria-label={`${text.deleteFile}: ${file.file_name}`}
            onClick={() => onRemove(file.id)}
          >
            <Trash2 aria-hidden="true" />
          </AttachmentAction>
        ) : null}
      </AttachmentActions>
    </Attachment>
  );
}
