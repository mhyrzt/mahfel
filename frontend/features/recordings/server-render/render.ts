import sharp from "sharp";
import { RenderAssets } from "./assets";
import { archivedImage, escapeXml, imageTag, pdfPage, textImage } from "./raster";
import { drawElement } from "./whiteboard";
import { RenderError, type RecordingRenderRequest, type RenderRun } from "./types";

function markup(run: RenderRun): string {
  let text = escapeXml(run.text);
  if (run.bold) {
    text = `<b>${text}</b>`;
  }
  if (run.italic) {
    text = `<i>${text}</i>`;
  }
  if (run.underline) {
    text = `<u>${text}</u>`;
  }
  if (run.strike) {
    text = `<s>${text}</s>`;
  }
  if (run.code) {
    text = `<tt>${text}</tt>`;
  }
  return text;
}

export async function renderImage(request: RecordingRenderRequest, signal: AbortSignal) {
  const { width, background, format } = request.output;
  const canvasHeight = request.output.height;
  const height = request.output.content_height ?? canvasHeight;
  const content = request.content;
  const assets = new RenderAssets(signal);
  let body = background ? `<rect width="${width}" height="${height}" fill="${background}"/>` : "";
  if (content.type === "whiteboard") {
    const { x, y, zoom } = content.viewport;
    let board = "";
    for (const element of content.elements) {
      signal.throwIfAborted();
      board += await drawElement(element, assets);
    }
    body += `<g transform="scale(${zoom}) translate(${x} ${y})">${board}</g>`;
  } else if (content.type === "notes") {
    const margin = 24;
    let y = margin - content.viewport.scroll_y;
    for (const block of content.document.blocks) {
      signal.throwIfAborted();
      if (y >= height) {
        break;
      }
      if (block.type === "image") {
        const image = await archivedImage(
          await assets.read(block.asset),
          Math.max(1, width - margin * 2),
          Math.min(360, height),
          "contain",
        );
        body += imageTag(image, margin, y, Math.max(1, width - margin * 2), Math.min(360, height));
        y += Math.min(360, height) + 12;
      } else {
        const size = block.type === "heading" ? Math.max(20, 36 - (block.level ?? 1) * 3) : 20;
        const prefix = block.type === "list_item" ? `${escapeXml(block.marker ?? "•")} ` : "";
        const direction = block.direction ?? content.direction;
        let align = block.align ?? "left";
        if (direction === "rtl" && align !== "center") {
          align = "right";
        }
        const text = await textImage(
          prefix + block.runs.map(markup).join(""),
          Math.max(1, width - margin * 2 - (block.type === "quote" ? 16 : 0)),
          size,
          align,
        );
        if (block.type === "quote") {
          body += `<rect x="${direction === "rtl" ? width - margin : margin}" y="${y}" width="3" height="${text.info.height}" fill="#888888"/>`;
        }
        body += imageTag(
          text.data,
          direction === "rtl"
            ? width - margin - text.info.width
            : margin + (block.type === "quote" ? 16 : 0),
          y,
          text.info.width,
          text.info.height,
        );
        y += text.info.height + 12;
      }
    }
  } else if (content.type === "slate") {
    const text = await textImage(
      `<span foreground="#f4f3f8">${escapeXml(content.text)}</span>`,
      Math.max(1, width - 48),
      32,
      "center",
    );
    body += imageTag(
      text.data,
      (width - text.info.width) / 2,
      (height - text.info.height) / 2,
      text.info.width,
      text.info.height,
    );
  } else {
    const bytes = await assets.read(content.asset);
    const image =
      content.type === "document_page"
        ? await pdfPage(bytes, content.page, width, height, content.fit, signal)
        : await archivedImage(bytes, width, height, content.fit);
    body += imageTag(image, 0, 0, width, height);
  }
  body = `<defs><clipPath id="content"><rect width="${width}" height="${height}"/></clipPath></defs><g clip-path="url(#content)">${body}</g>`;
  for (const annotation of request.annotations ?? []) {
    const text = await textImage(
      `<span foreground="${annotation.foreground}">${escapeXml(annotation.text)}</span>`,
      annotation.width - 16,
      annotation.font_size,
    );
    const panelHeight = Math.min(canvasHeight - annotation.y, text.info.height + 16);
    body += `<svg x="${annotation.x}" y="${annotation.y}" width="${annotation.width}" height="${Math.max(0, panelHeight)}"><rect width="100%" height="100%" rx="6" fill="${annotation.background}"/>${imageTag(text.data, 8, 8, text.info.width, text.info.height)}</svg>`;
  }
  const svg = Buffer.from(
    `<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="${width}" height="${canvasHeight}" viewBox="0 0 ${width} ${canvasHeight}">${body}</svg>`,
  );
  signal.throwIfAborted();
  const bytes =
    format === "svg"
      ? svg
      : await (
          format === "jpeg"
            ? sharp(svg)
                .flatten({ background: background ?? "#ffffff" })
                .jpeg({ quality: request.output.quality ?? 90 })
            : sharp(svg).png()
        ).toBuffer();
  if (bytes.length > 16 * 1024 * 1024) {
    throw new RenderError("output_resource_limit", 413);
  }
  return { bytes, mime: format === "svg" ? "image/svg+xml" : `image/${format}` };
}
