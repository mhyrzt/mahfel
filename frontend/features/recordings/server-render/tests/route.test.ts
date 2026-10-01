import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { NextRequest, NextResponse } from "next/server";
import sharp from "sharp";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { POST } from "@/app/api/internal/recording-render/route";
import { withRenderCapacity } from "../capacity";
import { parseRenderRequest } from "../validation";

const input = {
  version: 1,
  output: { format: "png", width: 640, height: 360, background: "#ffffff" },
  content: {
    type: "notes",
    direction: "rtl",
    document: {
      blocks: [
        {
          type: "paragraph",
          runs: [{ text: "یادداشت جلسه", bold: true }, { text: " <script>alert(1)</script>" }],
        },
      ],
    },
    viewport: { scroll_y: 0 },
  },
};

function request(body: unknown = input, authorization = "Bearer test-render-key") {
  return new NextRequest("http://localhost/api/internal/recording-render", {
    method: "POST",
    headers: { "content-type": "application/json", authorization },
    body: JSON.stringify(body),
  });
}

beforeEach(() => {
  vi.stubEnv("MAHFEL_RENDER_API_KEY", "test-render-key");
  vi.stubEnv("MAHFEL_RENDER_ASSET_ORIGINS", "http://storage.test");
});
afterEach(() => {
  vi.unstubAllEnvs();
  vi.unstubAllGlobals();
});

describe("native Next.js recording rendering", () => {
  it("renders empty workspace overlays and blank notes without invoking empty text shaping", async () => {
    for (const content of [
      { type: "slate", state: "empty", text: "" },
      {
        type: "notes",
        direction: "ltr",
        document: { blocks: [{ type: "paragraph", runs: [] }] },
        viewport: { scroll_y: 0 },
      },
    ]) {
      const response = await POST(request({ ...input, content }));
      expect(response.status).toBe(200);
      expect(await sharp(Buffer.from(await response.arrayBuffer())).metadata()).toMatchObject({
        width: 640,
        height: 360,
      });
    }
  });
  it("denies missing and incorrect keys before parsing or fetching input", async () => {
    const fetch = vi.fn();
    vi.stubGlobal("fetch", fetch);
    for (const key of ["", "Bearer wrong"]) {
      const response = await POST(request({}, key));
      expect(response).toBeInstanceOf(NextResponse);
      expect(response.status).toBe(401);
    }
    expect(fetch).not.toHaveBeenCalled();
  });

  it("fails closed when the rendering key is unconfigured", async () => {
    vi.stubEnv("MAHFEL_RENDER_API_KEY", "");
    expect((await POST(request())).status).toBe(503);
  });

  it.each(["png", "jpeg", "svg"])(
    "returns a native %s with text pixels and bounded dimensions",
    async (format) => {
      const response = await POST(request({ ...input, output: { ...input.output, format } }));
      expect(response.status).toBe(200);
      expect(response.headers.get("content-type")).toBe(
        format === "svg" ? "image/svg+xml" : `image/${format}`,
      );
      expect(response.headers.get("x-renderer-version")).toBe("native-v1");
      const bytes = Buffer.from(await response.arrayBuffer());
      if (format === "svg") {
        expect(bytes.toString()).not.toContain("<script>");
        expect(bytes.toString()).not.toMatch(/(?:href|src)="https?:/);
      }
      const image = sharp(bytes);
      expect(await image.metadata()).toMatchObject({ width: 640, height: 360 });
      const { data } = await image.removeAlpha().raw().toBuffer({ resolveWithObject: true });
      expect(data.some((value) => value < 200)).toBe(true);
    },
  );

  it("renders drawing paths, Persian whiteboard text and overlays without a DOM", async () => {
    const response = await POST(
      request({
        ...input,
        content: {
          type: "whiteboard",
          elements: [
            { type: "rectangle", x: 10, y: 20, width: 200, height: 100, fill: "#ff0000" },
            {
              type: "text",
              x: 30,
              y: 150,
              width: 300,
              height: 40,
              text: "سلام",
              stroke: "#000000",
            },
          ],
          viewport: { x: 0, y: 0, zoom: 1 },
        },
        annotations: [
          {
            text: "Participant",
            x: 0,
            y: 300,
            width: 200,
            font_size: 18,
            foreground: "#ffffff",
            background: "#101018",
          },
        ],
      }),
    );
    expect(response.status).toBe(200);
    const { data, info } = await sharp(Buffer.from(await response.arrayBuffer()))
      .raw()
      .toBuffer({ resolveWithObject: true });
    const pixel = (50 * info.width + 50) * info.channels;
    expect(data[pixel]).toBeGreaterThan(200);
    expect(data[pixel + 1]).toBeLessThan(50);
  });

  it("rejects oversized dimensions, injected colors, unsupported kinds and transparent JPEG", async () => {
    for (const value of [
      { ...input, version: 2 },
      { ...input, output: { ...input.output, width: 1921 } },
      { ...input, output: { ...input.output, background: "url(http://attacker)" } },
      { ...input, content: { type: "html", html: "<script/>" } },
      { ...input, output: { ...input.output, format: "jpeg", background: null } },
    ]) {
      expect((await POST(request(value))).status).toBe(422);
    }
    expect((await POST(request({ padding: "x".repeat(8 * 1024 * 1024) }))).status).toBe(413);
  });

  it("rejects external asset origins without contacting them", async () => {
    const fetch = vi.fn();
    vi.stubGlobal("fetch", fetch);
    const response = await POST(
      request({
        ...input,
        content: {
          type: "image",
          asset: { url: "http://attacker/asset", sha256: "0".repeat(64) },
          fit: "contain",
        },
      }),
    );
    expect(response.status).toBe(422);
    expect(fetch).not.toHaveBeenCalled();
  });

  it("verifies asset checksums and disables redirects", async () => {
    const bytes = await sharp({
      create: { width: 32, height: 32, channels: 4, background: "#008000" },
    })
      .png()
      .toBuffer();
    const fetch = vi.fn().mockImplementation(async () => new Response(bytes));
    vi.stubGlobal("fetch", fetch);
    const body = {
      ...input,
      content: {
        type: "image",
        asset: {
          url: "http://storage.test/asset",
          sha256: createHash("sha256").update(bytes).digest("hex"),
        },
        fit: "contain",
      },
    };
    expect((await POST(request(body))).status).toBe(200);
    expect(fetch).toHaveBeenCalledWith(
      expect.any(URL),
      expect.objectContaining({ redirect: "error" }),
    );
    body.content.asset.sha256 = "0".repeat(64);
    expect((await POST(request(body))).status).toBe(422);
  });

  it("rasterizes an archived PDF page with native Poppler", async () => {
    const bytes = await readFile(new URL("./fixtures/page.pdf", import.meta.url));
    vi.stubGlobal(
      "fetch",
      vi.fn().mockImplementation(async () => new Response(bytes)),
    );
    const response = await POST(
      request({
        ...input,
        content: {
          type: "document_page",
          asset: {
            url: "http://storage.test/deck.pdf",
            sha256: createHash("sha256").update(bytes).digest("hex"),
          },
          page: 1,
          fit: "contain",
        },
      }),
    );
    expect(response.status).toBe(200);
    expect(await sharp(Buffer.from(await response.arrayBuffer())).metadata()).toMatchObject({
      width: 640,
      height: 360,
    });
  });

  it("rejects excess concurrency and releases slots after failures", async () => {
    let release = () => {};
    const blocked = new Promise<void>((resolve) => {
      release = resolve;
    });
    const first = withRenderCapacity(() => blocked);
    const second = withRenderCapacity(() => blocked);
    const response = await POST(request());
    expect(response.status).toBe(503);
    expect(response.headers.get("retry-after")).toBe("1");
    release();
    await Promise.all([first, second]);
    await expect(
      withRenderCapacity(async () => {
        throw new Error("failure");
      }),
    ).rejects.toThrow("failure");
    expect((await POST(request())).status).toBe(200);
  });

  it("rejects malformed drawing points and excessive drawing work", () => {
    expect(() =>
      parseRenderRequest({
        ...input,
        content: {
          type: "whiteboard",
          elements: [{ type: "line", x: 0, y: 0, width: 1, height: 1, points: [[0, 0]] }],
          viewport: { x: 0, y: 0, zoom: 1 },
        },
      }),
    ).toThrow();
    expect(() =>
      parseRenderRequest({
        ...input,
        content: {
          type: "whiteboard",
          elements: [{ type: "rectangle", x: 0, y: 0, width: 20000, height: 20000 }],
          viewport: { x: 0, y: 0, zoom: 1 },
        },
      }),
    ).toThrow();
  });
});
