"use client";

import { useRef, useState, type DragEvent } from "react";
import { CloudUpload, FolderOpen, HardDriveUpload } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { listFiles, uploadFile } from "@/features/files/client";
import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";
import type { Locale } from "@/lib/i18n/config";

import { roomCopy } from "./room-copy";

type FileRecord = components["schemas"]["FileResponse"];

export function SessionFilePicker({
  sessionId,
  roomId,
  locale,
  attachedIds,
  onAdded,
  initialRoomFiles,
  attachRoomFile,
  uploadSessionFile,
}: Readonly<{
  sessionId: string;
  roomId: string;
  locale: Locale;
  attachedIds: string[];
  onAdded: (file: FileRecord) => void;
  initialRoomFiles?: FileRecord[];
  attachRoomFile?: (file: FileRecord) => Promise<FileRecord>;
  uploadSessionFile?: (file: File) => Promise<FileRecord>;
}>) {
  const text = roomCopy(locale);
  const inputRef = useRef<HTMLInputElement>(null);
  const [open, setOpen] = useState(false);
  const [bankOpen, setBankOpen] = useState(false);
  const [bankFiles, setBankFiles] = useState<FileRecord[]>(initialRoomFiles ?? []);
  const [loadingBank, setLoadingBank] = useState(false);
  const [pending, setPending] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [error, setError] = useState(false);

  async function upload(file: File) {
    setPending(true);
    setError(false);
    try {
      const added = uploadSessionFile
        ? await uploadSessionFile(file)
        : await uploadFile(file, { kind: "session", id: sessionId });
      onAdded(added);
      setOpen(false);
    } catch {
      setError(true);
    } finally {
      setPending(false);
    }
  }

  async function showBank() {
    setBankOpen(true);
    if (initialRoomFiles !== undefined) {
      return;
    }
    setLoadingBank(true);
    setError(false);
    try {
      setBankFiles(await listFiles({ kind: "room", id: roomId }, 100));
    } catch {
      setError(true);
    } finally {
      setLoadingBank(false);
    }
  }

  async function attach(file: FileRecord) {
    setPending(true);
    setError(false);
    try {
      const added = attachRoomFile
        ? await attachRoomFile(file)
        : (
            await browserApiClient.post<FileRecord>("/v1/files/session-links", {
              file_id: file.id,
              session_id: sessionId,
            })
          ).data;
      onAdded(added);
      setOpen(false);
      setBankOpen(false);
    } catch {
      setError(true);
    } finally {
      setPending(false);
    }
  }

  function drop(event: DragEvent<HTMLButtonElement>) {
    event.preventDefault();
    setDragging(false);
    const file = event.dataTransfer.files[0];
    if (file) {
      void upload(file);
    }
  }

  const available = bankFiles.filter((file) => !attachedIds.includes(file.id));
  return (
    <div className="space-y-2" data-testid={`session-file-picker-${sessionId}`}>
      <input
        accept="application/pdf,image/png,image/jpeg,video/mp4,video/webm"
        aria-label={text.uploadFromComputer}
        className="sr-only"
        disabled={pending}
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) {
            void upload(file);
          }
          event.target.value = "";
        }}
        ref={inputRef}
        type="file"
      />
      <Dialog
        onOpenChange={(value) => {
          setOpen(value);
          if (!value) {
            setBankOpen(false);
          }
        }}
        open={open}
      >
        <DialogTrigger asChild>
          <button
            className="flex min-h-28 w-full flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-border bg-background px-4 py-5 text-center text-sm transition-colors hover:border-primary/60 hover:bg-muted/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring data-[dragging=true]:border-primary data-[dragging=true]:bg-primary/5 disabled:opacity-50"
            data-dragging={dragging}
            disabled={pending}
            onDragEnter={(event) => {
              event.preventDefault();
              setDragging(true);
            }}
            onDragLeave={() => setDragging(false)}
            onDragOver={(event) => event.preventDefault()}
            onDrop={drop}
            type="button"
          >
            <CloudUpload aria-hidden="true" className="size-5 text-muted-foreground" />
            <span className="font-medium">{pending ? text.uploading : text.addSessionFile}</span>
            <span className="text-xs text-muted-foreground">{text.dropFileHint}</span>
          </button>
        </DialogTrigger>
        <DialogContent className="max-h-[min(85dvh,40rem)] max-w-lg overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{text.addSessionFile}</DialogTitle>
            <DialogDescription>{text.chooseFileSource}</DialogDescription>
          </DialogHeader>
          {bankOpen ? (
            <div className="space-y-3">
              <Button onClick={() => setBankOpen(false)} size="sm" type="button" variant="ghost">
                {text.back}
              </Button>
              {loadingBank ? (
                <p className="text-sm text-muted-foreground" role="status">
                  {text.loadingFiles}
                </p>
              ) : null}
              {!loadingBank && available.length === 0 ? (
                <p className="text-sm text-muted-foreground">{text.noRoomBankFiles}</p>
              ) : null}
              <ul className="max-h-72 space-y-1 overflow-y-auto">
                {available.map((file) => (
                  <li key={file.id}>
                    <Button
                      className="h-auto min-h-11 w-full justify-start whitespace-normal text-start"
                      disabled={pending}
                      onClick={() => void attach(file)}
                      type="button"
                      variant="ghost"
                    >
                      <FolderOpen aria-hidden="true" className="size-4 shrink-0" />
                      <span className="min-w-0 break-all">{file.file_name}</span>
                    </Button>
                  </li>
                ))}
              </ul>
            </div>
          ) : (
            <div className="grid gap-2 sm:grid-cols-2">
              <Button
                className="h-auto min-h-20 justify-start whitespace-normal text-start"
                onClick={() => {
                  setOpen(false);
                  inputRef.current?.click();
                }}
                type="button"
                variant="outline"
              >
                <HardDriveUpload aria-hidden="true" className="size-5" />
                {text.uploadFromComputer}
              </Button>
              <Button
                className="h-auto min-h-20 justify-start whitespace-normal text-start"
                onClick={() => void showBank()}
                type="button"
                variant="outline"
              >
                <FolderOpen aria-hidden="true" className="size-5" />
                {text.selectFromRoomBank}
              </Button>
            </div>
          )}
          {error ? (
            <p className="text-sm text-destructive" role="alert">
              {text.filePickFailed}
            </p>
          ) : null}
        </DialogContent>
      </Dialog>
      {!open && error ? (
        <p className="text-sm text-destructive" role="alert">
          {text.filePickFailed}
        </p>
      ) : null}
    </div>
  );
}
