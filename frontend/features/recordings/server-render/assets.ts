import { createHash } from "node:crypto";
import { RenderError, type ArchivedRenderAsset } from "./types";

const MAX_ASSET_BYTES = 50 * 1024 * 1024;

export class RenderAssets {
  private readonly loaded = new Map<string, Promise<Buffer>>();
  private bytes = 0;
  constructor(private readonly signal: AbortSignal) {}

  read(asset: ArchivedRenderAsset): Promise<Buffer> {
    const key = `${asset.sha256}:${asset.url}`;
    const existing = this.loaded.get(key);
    if (existing) {
      return existing;
    }
    if (this.loaded.size >= 32) {
      throw new RenderError("asset_resource_limit", 413);
    }
    const pending = this.download(asset);
    this.loaded.set(key, pending);
    return pending;
  }

  private async download(asset: ArchivedRenderAsset): Promise<Buffer> {
    let url: URL;
    try {
      url = new URL(asset.url);
    } catch {
      throw new RenderError("invalid_asset_url");
    }
    const origins = (process.env.MAHFEL_RENDER_ASSET_ORIGINS ?? "").split(",").filter(Boolean);
    if (
      !["http:", "https:"].includes(url.protocol) ||
      url.username ||
      url.password ||
      url.hash ||
      !origins.includes(url.origin)
    ) {
      throw new RenderError("asset_origin_forbidden");
    }
    let response: Response;
    try {
      response = await fetch(url, { redirect: "error", signal: this.signal, cache: "no-store" });
    } catch {
      throw new RenderError("asset_unavailable", 503);
    }
    if (!response.ok || !response.body) {
      throw new RenderError("asset_unavailable", 503);
    }
    const size = Number(response.headers.get("content-length"));
    if (size > MAX_ASSET_BYTES) {
      await response.body.cancel();
      throw new RenderError("asset_resource_limit", 413);
    }
    const reader = response.body.getReader();
    const chunks: Uint8Array[] = [];
    let count = 0;
    try {
      for (;;) {
        const chunk = await reader.read();
        if (chunk.done) {
          break;
        }
        count += chunk.value.byteLength;
        this.bytes += chunk.value.byteLength;
        if (count > MAX_ASSET_BYTES || this.bytes > 64 * 1024 * 1024) {
          throw new RenderError("asset_resource_limit", 413);
        }
        chunks.push(chunk.value);
      }
    } finally {
      await reader.cancel().catch(() => undefined);
    }
    const bytes = Buffer.concat(chunks);
    if (createHash("sha256").update(bytes).digest("hex") !== asset.sha256) {
      throw new RenderError("asset_checksum_mismatch");
    }
    return bytes;
  }
}
