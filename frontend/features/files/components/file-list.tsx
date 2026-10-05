"use client";

import { Empty, EmptyDescription } from "@/components/ui/empty";
import { useState } from "react";
import { LayoutGrid, List } from "lucide-react";

import { Button } from "@/components/ui/button";
import { roomCopy } from "@/features/rooms/room-copy";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { FileAttachment, canPreviewFile } from "./file-attachment";

type FileRecord = components["schemas"]["FileResponse"];
type FileView = "list" | "grid";

export { canPreviewFile };

export function FileList({
  files,
  locale,
  canDelete,
  actorId,
  onDownload,
  onPreview,
  onRemove,
  onPresent,
  initialView = "list",
  compact = false,
}: Readonly<{
  files: FileRecord[];
  locale: Locale;
  canDelete: boolean;
  actorId?: string;
  onDownload?: (fileId: string) => void;
  onPreview?: (file: FileRecord) => void;
  onRemove?: (fileId: string) => void;
  onPresent?: (fileId: string) => void;
  initialView?: FileView;
  compact?: boolean;
}>) {
  const text = roomCopy(locale);
  const [view, setView] = useState<FileView>(initialView);
  if (files.length === 0) {
    return (
      <Empty className="text-sm text-muted-foreground">
        <EmptyDescription>{text.noFiles}</EmptyDescription>
      </Empty>
    );
  }
  return (
    <div className="space-y-3">
      {!compact ? (
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
      ) : null}
      <ul
        className={
          view === "grid"
            ? "grid grid-cols-[repeat(auto-fit,minmax(min(100%,15rem),1fr))] gap-3"
            : "space-y-2"
        }
        data-testid={view === "grid" ? "file-grid" : "file-list"}
      >
        {files.map((file) => (
          <li key={file.id}>
            <FileAttachment
              file={file}
              locale={locale}
              onDownload={onDownload}
              onPreview={onPreview}
              onPresent={
                onPresent &&
                ["application/pdf", "image/png", "image/jpeg", "video/mp4", "video/webm"].includes(
                  file.mime,
                )
                  ? onPresent
                  : undefined
              }
              onRemove={
                canDelete || (actorId !== undefined && file.owner_id === actorId)
                  ? onRemove
                  : undefined
              }
              compact={compact}
            />
          </li>
        ))}
      </ul>
    </div>
  );
}
