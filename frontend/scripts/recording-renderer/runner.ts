import { chromium } from "playwright-core";
import { readFile, stat } from "node:fs/promises";
import { resolve, dirname, extname } from "node:path";
import { createInterface } from "node:readline";
import type { VideoTrack } from "../../features/recordings/replay/types";

const root = dirname(resolve(process.argv[1]));
const workspace = resolve(process.argv[2]);
const browser = await chromium.launch({
  executablePath: process.env.MAHFEL_RENDERER_CHROMIUM || undefined,
  chromiumSandbox: true,
  headless: true,
});
const page = await browser.newPage({
  viewport: { width: 1920, height: 1080 },
  deviceScaleFactor: 1,
  reducedMotion: "reduce",
  serviceWorkers: "block",
});
const mime: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".mjs": "text/javascript",
  ".css": "text/css",
  ".woff2": "font/woff2",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".webm": "video/webm",
  ".mp4": "video/mp4",
  ".pdf": "application/pdf",
};
const assets = JSON.parse(await readFile(`${workspace}/assets.json`, "utf8")) as Record<
  string,
  { path: string; mime: string }
>;
await page.route("**/*", async (route) => {
  const url = new URL(route.request().url());
  if (url.origin !== "https://recording.invalid") {
    await route.abort();
    return;
  }
  const path = decodeURIComponent(url.pathname);
  const archived = path.startsWith("/assets/") ? assets[path.slice(8)] : null;
  const file = archived
    ? resolve(workspace, archived.path)
    : resolve(root, path === "/" ? "index.html" : `.${path}`);
  const allowedRoot = archived ? workspace : root;
  if (!file.startsWith(`${allowedRoot}/`)) {
    await route.abort();
    return;
  }
  try {
    const info = await stat(file);
    if (info.size > 67108864) {
      throw new Error("Renderer asset too large");
    }
    await route.fulfill({
      body: await readFile(file),
      contentType: archived?.mime ?? mime[extname(file)] ?? "application/octet-stream",
    });
  } catch {
    await route.fulfill({ status: 404 });
  }
});
await page.goto("https://recording.invalid/");
await page.waitForFunction(() => Boolean(window.recordingRenderer));
let image = "";
const lines = createInterface({ input: process.stdin, crlfDelay: Infinity });
process.once("SIGTERM", () => {
  void browser.close().finally(() => process.exit(143));
});
try {
  for await (const line of lines) {
    if (line.length > 67108864) {
      throw new Error("Renderer command too large");
    }
    const command = JSON.parse(line) as {
      action: string;
      input?: unknown;
      timestamp?: string;
      tracks?: VideoTrack[];
      width?: number;
      height?: number;
    };
    if (command.action === "initialize") {
      await page.evaluate((input) => window.recordingRenderer.initialize(input), command.input);
      process.stdout.write('{"ready":true}\n');
    } else if (command.action === "render") {
      const width = command.width ?? 1920,
        height = command.height ?? 1080;
      if (width < 16 || height < 16 || width > 1920 || height > 1080) {
        throw new Error("Invalid render dimensions");
      }
      await page.setViewportSize({ width, height });
      const result = await page.evaluate(
        ({ timestamp, tracks, width, height }) =>
          window.recordingRenderer.render(timestamp, tracks, width, height),
        { timestamp: command.timestamp ?? "0", tracks: command.tracks ?? [], width, height },
      );
      if (!image || result.changed) {
        image = (
          await page.screenshot({ omitBackground: true, animations: "disabled", type: "png" })
        ).toString("base64");
      }
      process.stdout.write(`${JSON.stringify({ ...result, png: image })}\n`);
    } else if (command.action === "dispose") {
      break;
    } else {
      throw new Error("Unsupported renderer command");
    }
  }
} finally {
  lines.close();
  process.stdin.destroy();
  await browser.close();
}
