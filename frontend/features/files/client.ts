import { browserApiClient } from "@/lib/api/browser";
import type { components } from "@/lib/api/generated/schema";

type FileContextKind = components["schemas"]["FileContextKind"];
type FileResponse = components["schemas"]["FileResponse"];
type SignedFileResponse = components["schemas"]["SignedFileResponse"];

export interface FileContext {
  kind: FileContextKind;
  id: string;
}

function sha256Base64(bytes: ArrayBuffer): Promise<string> {
  return crypto.subtle.digest("SHA-256", bytes).then((digest) => {
    const raw = new Uint8Array(digest);
    let binary = "";
    for (const byte of raw) {
      binary += String.fromCharCode(byte);
    }
    return btoa(binary);
  });
}

export async function uploadFile(file: File, context: FileContext): Promise<FileResponse> {
  const checksum = await sha256Base64(await file.arrayBuffer());
  const initiated = await browserApiClient.post<SignedFileResponse>("/v1/files/uploads", {
    context_kind: context.kind,
    context_id: context.id,
    file_name: file.name,
    mime: file.type,
    size_bytes: file.size,
    checksum_sha256: checksum,
  });
  const { url, headers, file: pending } = initiated.data;
  const uploadHeaders = new Headers();
  for (const { name, value } of headers) {
    if (name.toLowerCase() !== "host" && name.toLowerCase() !== "content-length") {
      uploadHeaders.set(name, value);
    }
  }
  const transferred = await fetch(url, {
    method: "PUT",
    body: file,
    headers: uploadHeaders,
    credentials: "omit",
    redirect: "error",
  });
  if (!transferred.ok) {
    throw new Error(`File transfer failed (${transferred.status})`);
  }
  const completed = await browserApiClient.post<FileResponse>(`/v1/files/${pending.id}/complete`);
  return completed.data;
}

export async function listFiles(
  context: FileContext,
  limit = 50,
  offset = 0,
): Promise<FileResponse[]> {
  const response = await browserApiClient.get<{ files: FileResponse[] }>("/v1/files", {
    params: { context_kind: context.kind, context_id: context.id, limit, offset },
  });
  return response.data.files;
}

export async function getFileDownloadUrl(fileId: string): Promise<string> {
  const response = await browserApiClient.get<SignedFileResponse>(`/v1/files/${fileId}/download`);
  return response.data.url;
}
