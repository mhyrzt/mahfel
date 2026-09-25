import type { BinaryFileData } from "@excalidraw/excalidraw/types";
import type { FileId } from "@excalidraw/excalidraw/element/types";

import { uploadFile } from "@/features/files/client";
import { browserApiClient } from "@/lib/api/browser";

type DocumentKind = "notes" | "whiteboard";
type DownloadResponse = { url: string; file: { mime: string } };

export function isSupportedCollaborationImage(mime: string): mime is "image/png" | "image/jpeg" {
  return mime === "image/png" || mime === "image/jpeg";
}

export async function registerCollaborationImage(
  file: File,
  sessionId: string,
  kind: DocumentKind,
): Promise<string> {
  if (!isSupportedCollaborationImage(file.type)) {
    throw new Error("Only PNG and JPEG images are supported");
  }
  const uploaded = await uploadFile(file, { kind: "session", id: sessionId });
  await browserApiClient.post(
    `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/${kind}/images`,
    { file_id: uploaded.id },
  );
  return uploaded.id;
}

export async function fetchCollaborationImage(fileId: string): Promise<Blob> {
  const { data } = await browserApiClient.get<DownloadResponse>(
    `/v1/files/${encodeURIComponent(fileId)}/download`,
  );
  if (!isSupportedCollaborationImage(data.file.mime)) {
    throw new Error("Unsupported collaboration image");
  }
  const url = new URL(data.url);
  if (url.protocol !== "https:" && url.protocol !== "http:") {
    throw new Error("Unsupported download URL");
  }
  const response = await fetch(url, { credentials: "omit", redirect: "error" });
  if (!response.ok) {
    throw new Error("Image download failed");
  }
  return new Blob([await response.blob()], { type: data.file.mime });
}

export async function excalidrawImageData(
  fileId: string,
  imageId: FileId,
): Promise<BinaryFileData> {
  const blob = await fetchCollaborationImage(fileId);
  const dataURL = await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(blob);
  });
  return {
    id: imageId,
    dataURL: dataURL as BinaryFileData["dataURL"],
    mimeType: blob.type as BinaryFileData["mimeType"],
    created: Date.now(),
  };
}
