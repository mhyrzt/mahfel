import type { NextRequest } from "next/server";
import { RenderError, type RecordingRenderRequest } from "./types";
import { parseRenderRequest } from "./validation";

async function readBody(request: NextRequest, signal: AbortSignal): Promise<Buffer> {
  if (!request.body) {
    throw new RenderError("invalid_input");
  }
  const reader = request.body.getReader();
  const abort = () => {
    void reader.cancel().catch(() => undefined);
  };
  signal.addEventListener("abort", abort, { once: true });
  const chunks: Uint8Array[] = [];
  let size = 0;
  try {
    for (;;) {
      signal.throwIfAborted();
      const chunk = await reader.read();
      signal.throwIfAborted();
      if (chunk.done) {
        return Buffer.concat(chunks);
      }
      size += chunk.value.byteLength;
      if (size > 8 * 1024 * 1024) {
        throw new RenderError("request_resource_limit", 413);
      }
      chunks.push(chunk.value);
    }
  } finally {
    signal.removeEventListener("abort", abort);
    await reader.cancel().catch(() => undefined);
  }
}

export async function readRenderRequest(
  request: NextRequest,
  signal: AbortSignal,
): Promise<RecordingRenderRequest> {
  if (request.headers.get("content-type")?.split(";")[0] !== "application/json") {
    throw new RenderError("unsupported_content_type", 415);
  }
  const bytes = await readBody(request, signal);
  let value: unknown;
  try {
    value = JSON.parse(bytes.toString());
  } catch {
    throw new RenderError("invalid_json");
  }
  return parseRenderRequest(value);
}
