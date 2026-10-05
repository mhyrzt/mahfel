"use client";

import { Button } from "@/components/ui/button";
import { useEffect, useState } from "react";
import Image from "next/image";
import { FileVideo, Play } from "lucide-react";

import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { FileAttachment, canPreviewFile } from "@/features/files/components/file-attachment";
import { getFileDownloadUrl } from "@/features/files/client";
import { roomCopy } from "@/features/rooms/room-copy";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

type FileRecord = components["schemas"]["FileResponse"];

export function isMediaFile(mime: string): boolean {
  return mime.startsWith("image/") || mime.startsWith("video/");
}

function useObjectUrl(fileId: string): string | null {
  const [url, setUrl] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    let objectUrl: string | null = null;
    void (async () => {
      try {
        const signed = new URL(await getFileDownloadUrl(fileId), window.location.origin);
        if (!["http:", "https:"].includes(signed.protocol)) {
          return;
        }
        const response = await fetch(signed, { credentials: "omit", redirect: "error" });
        if (!response.ok) {
          return;
        }
        objectUrl = URL.createObjectURL(await response.blob());
        if (active) {
          setUrl(objectUrl);
        } else {
          URL.revokeObjectURL(objectUrl);
        }
      } catch {}
    })();
    return () => {
      active = false;
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    };
  }, [fileId]);
  return url;
}

function useVideoPoster(fileId: string): string | null {
  const [poster, setPoster] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    let objectUrl: string | null = null;
    void (async () => {
      try {
        const signed = new URL(await getFileDownloadUrl(fileId), window.location.origin);
        if (!["http:", "https:"].includes(signed.protocol)) {
          return;
        }
        const response = await fetch(signed, { credentials: "omit", redirect: "error" });
        if (!response.ok) {
          return;
        }
        objectUrl = URL.createObjectURL(await response.blob());
        const video = document.createElement("video");
        video.muted = true;
        video.preload = "metadata";
        video.src = objectUrl;
        await new Promise<void>((resolve, reject) => {
          video.onloadeddata = () => resolve();
          video.onerror = () => reject(new Error("video load failed"));
        });
        const duration = Number.isFinite(video.duration) ? video.duration : 1;
        await new Promise<void>((resolve) => {
          video.onseeked = () => resolve();
          video.currentTime = Math.min(0.5, Math.max(0, duration / 2));
        });
        const width = video.videoWidth || 640;
        const height = video.videoHeight || 360;
        const scale = Math.min(1, 640 / width);
        const canvas = document.createElement("canvas");
        canvas.width = Math.round(width * scale);
        canvas.height = Math.round(height * scale);
        canvas.getContext("2d")?.drawImage(video, 0, 0, canvas.width, canvas.height);
        const posterUrl = canvas.toDataURL("image/jpeg", 0.72);
        if (active) {
          setPoster(posterUrl);
        }
      } catch {}
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    })();
    return () => {
      active = false;
    };
  }, [fileId]);
  return poster;
}

function ImageTile({
  file,
  onOpen,
}: Readonly<{ file: FileRecord; onOpen: (file: FileRecord) => void }>) {
  const url = useObjectUrl(file.id);
  return (
    <Button
      variant="plain"
      aria-label={file.file_name}
      className="group/media relative block h-auto w-full p-0 aspect-square min-h-24 overflow-hidden rounded-lg border bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={() => onOpen(file)}
      type="button"
    >
      {url ? (
        <Image
          alt={file.file_name}
          className="size-full object-cover transition-transform group-hover/media:scale-105"
          height={240}
          src={url}
          unoptimized
          width={240}
        />
      ) : (
        <span className="block size-full" />
      )}
    </Button>
  );
}

function VideoTile({
  file,
  onOpen,
}: Readonly<{ file: FileRecord; onOpen: (file: FileRecord) => void }>) {
  const poster = useVideoPoster(file.id);
  return (
    <Button
      variant="plain"
      aria-label={file.file_name}
      className="relative block h-auto w-full p-0 aspect-video min-h-24 overflow-hidden rounded-lg border bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={() => onOpen(file)}
      type="button"
    >
      {poster ? (
        <Image
          alt={file.file_name}
          className="size-full object-cover"
          height={360}
          src={poster}
          unoptimized
          width={640}
        />
      ) : (
        <span className="grid size-full place-items-center text-muted-foreground">
          <FileVideo aria-hidden="true" className="size-6" />
        </span>
      )}
      <span className="absolute inset-0 grid place-items-center bg-black/25">
        <Play aria-hidden="true" className="size-6 fill-white text-white" />
      </span>
    </Button>
  );
}

function PreviewBody({ file }: Readonly<{ file: FileRecord }>) {
  const url = useObjectUrl(file.id);
  if (!url) {
    return null;
  }
  if (file.mime.startsWith("video/")) {
    return <video className="mx-auto max-h-[70dvh] w-full rounded-md" controls src={url} />;
  }
  if (file.mime.startsWith("image/")) {
    return (
      <Image
        alt={file.file_name}
        className="mx-auto h-auto max-h-[70dvh] w-auto max-w-full object-contain"
        height={900}
        src={url}
        unoptimized
        width={1200}
      />
    );
  }
  return (
    <iframe
      className="h-[70dvh] w-full rounded-md border"
      referrerPolicy="no-referrer"
      sandbox="allow-scripts"
      src={url}
      title={file.file_name}
    />
  );
}

export function MessageAttachments({
  files,
  locale,
}: Readonly<{ files: FileRecord[]; locale: Locale }>) {
  const text = roomCopy(locale);
  const [preview, setPreview] = useState<FileRecord | null>(null);

  if (files.length === 0) {
    return null;
  }
  const images = files.filter((file) => file.mime.startsWith("image/"));
  const videos = files.filter((file) => file.mime.startsWith("video/"));
  const media = [...images, ...videos];
  const documents = files.filter((file) => !isMediaFile(file.mime));

  async function download(fileId: string) {
    window.location.assign(await getFileDownloadUrl(fileId));
  }

  return (
    <div className="mt-1.5 space-y-1.5" data-testid="message-attachments">
      {media.length > 0 ? (
        <div
          className={`grid max-w-[24rem] gap-1 ${media.length > 1 ? "grid-cols-2" : "grid-cols-1"}`}
          data-testid="message-media-grid"
        >
          {media.map((file) =>
            file.mime.startsWith("video/") ? (
              <VideoTile file={file} key={file.id} onOpen={setPreview} />
            ) : (
              <ImageTile file={file} key={file.id} onOpen={setPreview} />
            ),
          )}
        </div>
      ) : null}
      {documents.map((file) => (
        <FileAttachment
          compact
          file={file}
          key={file.id}
          locale={locale}
          onDownload={(id) => void download(id)}
          onPreview={canPreviewFile(file.mime) ? setPreview : undefined}
        />
      ))}
      <Dialog onOpenChange={(open) => !open && setPreview(null)} open={preview !== null}>
        <DialogContent className="max-h-[calc(100dvh-2rem)] max-w-4xl overflow-y-auto">
          <DialogTitle>{preview?.file_name}</DialogTitle>
          <DialogDescription className="sr-only">{text.preview}</DialogDescription>
          {preview ? <PreviewBody file={preview} /> : null}
        </DialogContent>
      </Dialog>
    </div>
  );
}
