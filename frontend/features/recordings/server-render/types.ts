export interface ArchivedRenderAsset {
  url: string;
  sha256: string;
}

export interface RenderRun {
  text: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strike?: boolean;
  code?: boolean;
}

export type RenderBlock =
  | {
      type: "paragraph" | "heading" | "quote" | "list_item";
      runs: RenderRun[];
      level?: number;
      direction?: "ltr" | "rtl";
      marker?: string;
      align?: "left" | "center" | "right";
    }
  | { type: "image"; asset: ArchivedRenderAsset; alt: string };

export interface RenderElement {
  type:
    | "rectangle"
    | "ellipse"
    | "diamond"
    | "arrow"
    | "line"
    | "freedraw"
    | "text"
    | "frame"
    | "image";
  x: number;
  y: number;
  width: number;
  height: number;
  angle: number;
  stroke: string;
  fill: string;
  stroke_width: number;
  opacity: number;
  seed: number;
  roughness: number;
  fill_style: "solid" | "hachure" | "cross-hatch";
  stroke_style: "solid" | "dashed" | "dotted";
  points?: [number, number][];
  text?: string;
  font_size?: number;
  align?: "left" | "center" | "right";
  start_arrowhead?: "arrow" | "bar" | "dot" | "triangle" | "diamond" | null;
  end_arrowhead?: "arrow" | "bar" | "dot" | "triangle" | "diamond" | null;
  asset?: ArchivedRenderAsset;
}

export type RenderContent =
  | {
      type: "whiteboard";
      elements: RenderElement[];
      viewport: { x: number; y: number; zoom: number };
    }
  | {
      type: "notes";
      direction: "ltr" | "rtl";
      document: { blocks: RenderBlock[] };
      viewport: { scroll_y: number };
    }
  | { type: "document_page"; asset: ArchivedRenderAsset; page: number; fit: "contain" | "cover" }
  | { type: "image"; asset: ArchivedRenderAsset; fit: "contain" | "cover" }
  | { type: "slate"; state: "paused" | "outage" | "empty"; text: string };

export interface RecordingRenderRequest {
  version: 1;
  output: {
    format: "png" | "jpeg" | "svg";
    width: number;
    height: number;
    content_height?: number;
    background: string | null;
    quality?: number;
  };
  content: RenderContent;
  annotations?: {
    text: string;
    x: number;
    y: number;
    width: number;
    font_size: number;
    foreground: string;
    background: string;
  }[];
}

export class RenderError extends Error {
  constructor(
    readonly code: string,
    readonly status = 422,
  ) {
    super(code);
  }
}
