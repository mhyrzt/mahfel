import {
  RenderError,
  type ArchivedRenderAsset,
  type RecordingRenderRequest,
  type RenderBlock,
  type RenderContent,
  type RenderElement,
  type RenderRun,
} from "./types";

function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new RenderError("invalid_input");
  }
  return value as Record<string, unknown>;
}
function number(value: unknown, min: number, max: number, fallback?: number): number {
  if (value === undefined && fallback !== undefined) {
    return fallback;
  }
  if (typeof value !== "number" || !Number.isFinite(value) || value < min || value > max) {
    throw new RenderError("invalid_number");
  }
  return value;
}
function integer(value: unknown, min: number, max: number, fallback?: number): number {
  const parsed = number(value, min, max, fallback);
  if (!Number.isInteger(parsed)) {
    throw new RenderError("invalid_number");
  }
  return parsed;
}
function string(value: unknown, max = 10000): string {
  if (typeof value !== "string" || value.length > max) {
    throw new RenderError("invalid_string");
  }
  return value;
}
function choice<const T extends string>(value: unknown, options: readonly T[], fallback?: T): T {
  if (value === undefined && fallback) {
    return fallback;
  }
  if (typeof value !== "string" || !options.includes(value as T)) {
    throw new RenderError("unsupported_content");
  }
  return value as T;
}
function array(value: unknown, max: number): unknown[] {
  if (!Array.isArray(value) || value.length > max) {
    throw new RenderError("resource_limit");
  }
  return value;
}
export function color(value: unknown, fallback?: string): string {
  if (value === undefined && fallback) {
    return fallback;
  }
  const parsed = string(value, 16);
  if (parsed !== "transparent" && !/^#[a-fA-F0-9]{6}([a-fA-F0-9]{2})?$/.test(parsed)) {
    throw new RenderError("invalid_color");
  }
  return parsed;
}
function asset(value: unknown): ArchivedRenderAsset {
  const data = object(value);
  const url = string(data.url, 4096);
  const sha256 = string(data.sha256, 64);
  if (!/^[a-f0-9]{64}$/.test(sha256)) {
    throw new RenderError("invalid_checksum");
  }
  return { url, sha256 };
}
function run(value: unknown): RenderRun {
  const data = object(value);
  for (const key of ["bold", "italic", "underline", "strike", "code"]) {
    if (data[key] !== undefined && typeof data[key] !== "boolean") {
      throw new RenderError("invalid_text_format");
    }
  }
  return {
    text: string(data.text),
    bold: data.bold === true,
    italic: data.italic === true,
    underline: data.underline === true,
    strike: data.strike === true,
    code: data.code === true,
  };
}
function block(value: unknown): RenderBlock {
  const data = object(value);
  if (data.type === "image") {
    return { type: "image", asset: asset(data.asset), alt: string(data.alt, 1024) };
  }
  return {
    type: choice(data.type, ["paragraph", "heading", "quote", "list_item"]),
    runs: array(data.runs, 500).map(run),
    level: integer(data.level, 1, 6, 1),
    direction: data.direction === undefined ? undefined : choice(data.direction, ["ltr", "rtl"]),
    marker: data.marker === undefined ? undefined : string(data.marker, 32),
    align: choice(data.align, ["left", "center", "right"], "left"),
  };
}
function element(value: unknown): RenderElement {
  const data = object(value);
  const type = choice(data.type, [
    "rectangle",
    "ellipse",
    "diamond",
    "arrow",
    "line",
    "freedraw",
    "text",
    "frame",
    "image",
  ]);
  const parsed: RenderElement = {
    type,
    x: number(data.x, -1000000, 1000000),
    y: number(data.y, -1000000, 1000000),
    width: number(data.width, 0, 20000),
    height: number(data.height, 0, 20000),
    angle: number(data.angle, -100, 100, 0),
    stroke: color(data.stroke, "#1e1e1e"),
    fill: color(data.fill, "transparent"),
    stroke_width: number(data.stroke_width, 0, 20, 1),
    opacity: number(data.opacity, 0, 1, 1),
    seed: integer(data.seed, 1, 2147483647, 1),
    roughness: number(data.roughness, 0, 3, 1),
    fill_style: choice(data.fill_style, ["solid", "hachure", "cross-hatch"], "solid"),
    stroke_style: choice(data.stroke_style, ["solid", "dashed", "dotted"], "solid"),
  };
  if (type === "text") {
    parsed.text = string(data.text);
    parsed.font_size = number(data.font_size, 1, 512, 20);
    parsed.align = choice(data.align, ["left", "center", "right"], "left");
  }
  if (["line", "arrow", "freedraw"].includes(type)) {
    parsed.points = array(data.points, 10000).map((value) => {
      const point = array(value, 2);
      if (point.length !== 2) {
        throw new RenderError("invalid_point");
      }
      return [number(point[0], -20000, 20000), number(point[1], -20000, 20000)];
    });
    if (parsed.points.length < 2) {
      throw new RenderError("invalid_point");
    }
    for (const key of ["start_arrowhead", "end_arrowhead"] as const) {
      parsed[key] =
        data[key] == null
          ? null
          : choice(data[key], ["arrow", "bar", "dot", "triangle", "diamond"]);
    }
  }
  if (type === "image") {
    parsed.asset = asset(data.asset);
  }
  return parsed;
}
function content(value: unknown): RenderContent {
  const data = object(value);
  switch (data.type) {
    case "whiteboard": {
      const viewport = object(data.viewport);
      const elements = array(data.elements, 2000).map(element);
      if (elements.reduce((area, item) => area + item.width * item.height, 0) > 16000000) {
        throw new RenderError("resource_limit");
      }
      if (elements.reduce((count, item) => count + (item.points?.length ?? 0), 0) > 20000) {
        throw new RenderError("resource_limit");
      }
      return {
        type: "whiteboard",
        elements,
        viewport: {
          x: number(viewport.x, -1000000, 1000000),
          y: number(viewport.y, -1000000, 1000000),
          zoom: number(viewport.zoom, 0.01, 10),
        },
      };
    }
    case "notes": {
      const viewport = object(data.viewport);
      const blocks = array(object(data.document).blocks, 1000).map(block);
      const length = blocks.reduce(
        (sum, item) =>
          sum +
          (item.type === "image"
            ? 0
            : item.runs.reduce((count, run) => count + run.text.length, 0)),
        0,
      );
      if (length > 100000) {
        throw new RenderError("resource_limit");
      }
      return {
        type: "notes",
        direction: choice(data.direction, ["ltr", "rtl"]),
        document: { blocks },
        viewport: { scroll_y: number(viewport.scroll_y, 0, 1000000) },
      };
    }
    case "document_page":
      return {
        type: "document_page",
        asset: asset(data.asset),
        page: integer(data.page, 1, 10000),
        fit: choice(data.fit, ["contain", "cover"], "contain"),
      };
    case "image":
      return {
        type: "image",
        asset: asset(data.asset),
        fit: choice(data.fit, ["contain", "cover"], "contain"),
      };
    case "slate":
      return {
        type: "slate",
        state: choice(data.state, ["paused", "outage", "empty"]),
        text: string(data.text, 1024),
      };
    default:
      throw new RenderError("unsupported_content");
  }
}
export function parseRenderRequest(value: unknown): RecordingRenderRequest {
  const data = object(value);
  if (data.version !== 1) {
    throw new RenderError("unsupported_version");
  }
  const output = object(data.output);
  const format = choice(output.format, ["png", "jpeg", "svg"]);
  const background = output.background == null ? null : color(output.background);
  if (
    format === "jpeg" &&
    (!background || background === "transparent" || background.length !== 7)
  ) {
    throw new RenderError("jpeg_requires_opaque_background");
  }
  return {
    version: 1,
    output: {
      format,
      width: integer(output.width, 16, 1920),
      height: integer(output.height, 16, 1080),
      content_height:
        output.content_height === undefined
          ? undefined
          : integer(output.content_height, 16, integer(output.height, 16, 1080)),
      background,
      quality: output.quality === undefined ? undefined : integer(output.quality, 1, 100),
    },
    content: content(data.content),
    annotations:
      data.annotations === undefined
        ? undefined
        : array(data.annotations, 16).map((value) => {
            const annotation = object(value);
            return {
              text: string(annotation.text, 1024),
              x: number(annotation.x, 0, 1920),
              y: number(annotation.y, 0, 1080),
              width: integer(annotation.width, 16, 1920),
              font_size: number(annotation.font_size, 8, 48),
              foreground: color(annotation.foreground),
              background: color(annotation.background),
            };
          }),
  };
}
