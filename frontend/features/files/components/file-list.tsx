"use client";

import { useState } from "react";
import { Download, Eye, File, FileImage, FileText, LayoutGrid, List, Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { roomCopy } from "@/features/rooms/room-copy";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

type FileRecord = components["schemas"]["FileResponse"];
type FileView = "list" | "grid";

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

function iconFor(mime: string) {
  if (mime.startsWith("image/")) {
    return FileImage;
  }
  if (mime === "application/pdf" || mime.startsWith("text/")) {
    return FileText;
  }
  return File;
}

export function FileList({
  files,
  locale,
  canDelete,
  actorId,
  onDownload,
  onPreview,
  onRemove,
  initialView = "list",
}: Readonly<{
  files: FileRecord[];
  locale: Locale;
  canDelete: boolean;
  actorId?: string;
  onDownload?: (fileId: string) => void;
  onPreview?: (file: FileRecord) => void;
  onRemove?: (fileId: string) => void;
  initialView?: FileView;
}>) {
  const text = roomCopy(locale);
  const [view, setView] = useState<FileView>(initialView);
  if (files.length === 0) {
    return <p className="text-sm text-muted-foreground">{text.noFiles}</p>;
  }
  return (
    <div className="space-y-3">
      <div aria-label={text.fileView} className="flex justify-end gap-1" role="group">
        <Button
          aria-label={text.listView}
          aria-pressed={view === "list"}
          onClick={() => setView("list")}
          size="icon"
          variant={view === "list" ? "secondary" : "ghost"}
        >
          <List aria-hidden="true" className="size-4" />
        </Button>
        <Button
          aria-label={text.gridView}
          aria-pressed={view === "grid"}
          onClick={() => setView("grid")}
          size="icon"
          variant={view === "grid" ? "secondary" : "ghost"}
        >
          <LayoutGrid aria-hidden="true" className="size-4" />
        </Button>
      </div>
      <ul
        className={
          view === "grid" ? "grid gap-3 sm:grid-cols-2 lg:grid-cols-3" : "divide-y border-y"
        }
        data-testid={view === "grid" ? "file-grid" : "file-list"}
      >
        {files.map((file) => {
          const Icon = iconFor(file.mime);
          const mayDelete = canDelete || (actorId !== undefined && file.owner_id === actorId);
          return (
            <li
              className={
                view === "grid"
                  ? "flex min-w-0 flex-col gap-4 rounded-xl border bg-card p-4"
                  : "flex min-h-14 items-center gap-3 py-2"
              }
              key={file.id}
            >
              <div
                className={
                  view === "grid"
                    ? "flex min-w-0 items-start gap-3"
                    : "flex min-w-0 flex-1 items-center gap-3"
                }
              >
                <Icon
                  aria-hidden="true"
                  className="size-5 shrink-0 text-muted-foreground"
                  strokeWidth={1.7}
                />
                <span className="min-w-0 truncate text-sm font-medium" title={file.file_name}>
                  {file.file_name}
                </span>
              </div>
              <div className={view === "grid" ? "flex justify-end gap-1" : "flex shrink-0 gap-1"}>
                <Button
                  aria-label={`${text.preview}: ${file.file_name}`}
                  disabled={!canPreviewFile(file.mime) || !onPreview}
                  onClick={() => onPreview?.(file)}
                  size="icon"
                  title={canPreviewFile(file.mime) ? text.preview : text.previewUnavailable}
                  variant="ghost"
                >
                  <Eye aria-hidden="true" className="size-4" />
                </Button>
                <Button
                  aria-label={`${text.download}: ${file.file_name}`}
                  onClick={() => onDownload?.(file.id)}
                  size="icon"
                  variant="ghost"
                >
                  <Download aria-hidden="true" className="size-4" />
                </Button>
                {mayDelete ? (
                  <Button
                    aria-label={`${text.deleteFile}: ${file.file_name}`}
                    onClick={() => onRemove?.(file.id)}
                    size="icon"
                    variant="ghost"
                  >
                    <Trash2 aria-hidden="true" className="size-4" />
                  </Button>
                ) : null}
              </div>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
