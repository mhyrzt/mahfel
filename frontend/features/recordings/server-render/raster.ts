import sharp from "sharp";
import { execFile } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { promisify } from "node:util";
import { RenderError } from "./types";

const execute = promisify(execFile);
const MAX_PIXELS = 16 * 1024 * 1024;
const fontFile = resolve(
  process.cwd(),
  "features/recordings/server-render/assets/Vazirmatn-Regular.ttf",
);

export function escapeXml(value: string): string {
  return value.replace(
    /[&<>"']/g,
    (char) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&apos;" })[char] ?? "",
  );
}

export async function textImage(
  markup: string,
  width: number,
  size: number,
  align: "left" | "center" | "right" = "left",
) {
  if (!markup.replace(/<[^>]*>/g, "").trim()) {
    return sharp({
      create: {
        width: 1,
        height: Math.ceil(size * markup.split("\n").length),
        channels: 4,
        background: "#00000000",
      },
    })
      .png()
      .toBuffer({ resolveWithObject: true });
  }
  return sharp({
    text: {
      text: markup || " ",
      font: `Vazirmatn ${size}`,
      fontfile: fontFile,
      width: Math.max(1, Math.ceil(width)),
      dpi: 72,
      rgba: true,
      align,
      wrap: "word-char",
    },
    limitInputPixels: MAX_PIXELS,
  })
    .png()
    .toBuffer({ resolveWithObject: true });
}

export async function archivedImage(
  bytes: Buffer,
  width: number,
  height: number,
  fit: "contain" | "cover",
) {
  const image = sharp(bytes, { limitInputPixels: MAX_PIXELS });
  const metadata = await image.metadata();
  if (!["png", "jpeg"].includes(metadata.format ?? "")) {
    throw new RenderError("unsupported_asset_format");
  }
  return image
    .rotate()
    .resize(Math.max(1, Math.ceil(width)), Math.max(1, Math.ceil(height)), {
      fit,
      background: "#00000000",
    })
    .png()
    .toBuffer();
}

export async function pdfPage(
  bytes: Buffer,
  page: number,
  width: number,
  height: number,
  fit: "contain" | "cover",
  signal: AbortSignal,
) {
  if (bytes.subarray(0, 5).toString() !== "%PDF-") {
    throw new RenderError("unsupported_asset_format");
  }
  const directory = await mkdtemp(join(tmpdir(), "mahfel-render-"));
  try {
    await writeFile(join(directory, "input.pdf"), bytes);
    try {
      await execute(
        "pdftocairo",
        [
          "-png",
          "-singlefile",
          "-f",
          String(page),
          "-l",
          String(page),
          "-scale-to",
          String(Math.max(width, height)),
          join(directory, "input.pdf"),
          join(directory, "page"),
        ],
        { timeout: 15000, signal, maxBuffer: 65536 },
      );
    } catch {
      throw new RenderError("document_render_failed");
    }
    return archivedImage(await readFile(join(directory, "page.png")), width, height, fit);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

export function imageTag(
  bytes: Buffer,
  x: number,
  y: number,
  width: number,
  height: number,
): string {
  return `<image x="${x}" y="${y}" width="${width}" height="${height}" href="data:image/png;base64,${bytes.toString("base64")}"/>`;
}
