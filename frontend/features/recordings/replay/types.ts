export type Surface = "notes" | "whiteboard" | "presentation";
export type Row = Record<string, unknown>;
export interface ReplaySnapshot {
  generation: number;
  snapshot_kind: string;
  source_id: string;
  payload: Row;
}
export interface ReplayEvent {
  generation: number;
  event_kind: string;
  source_id: string;
  at_ns: string;
  payload: Row;
}
export interface ReplayInterval {
  generation: number;
  start_ns: string;
  end_ns: string;
  loss_reason?: string | null;
}
export interface ReplayAsset {
  file_id: string;
  mime: string | null;
  archive_key: string;
}
export interface ReplayInput {
  partial?: boolean;
  version: 1;
  start_ns: string;
  duration_ns: string;
  snapshots: ReplaySnapshot[];
  events: ReplayEvent[];
  intervals: ReplayInterval[];
  assets: ReplayAsset[];
  actors: Record<string, string>;
}
export interface Toast {
  id: string;
  author: string;
  body: string;
  expiresNs: bigint;
}
export interface Presentation {
  fileIds: string[];
  page: number;
  positionMs: number;
  paused: boolean;
  rate: number;
  anchorNs: bigint;
}
export interface VideoTrack {
  started_ns?: string;
  id: string;
  actorId: string;
  source: "camera" | "screen";
}
export interface SceneLayer {
  source: string;
  x: number;
  y: number;
  width: number;
  height: number;
  fit: "contain" | "cover";
  z: number;
}
export interface SceneV1 {
  version: 1;
  timestamp_ns: string;
  canvas: { width: number; height: number };
  surface: Surface | null;
  slate: "paused" | "outage" | null;
  layers: SceneLayer[];
  cameraPage: number;
  cameraPages: number;
}

export function row(value: unknown): Row {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("Invalid replay row");
  }
  return value as Row;
}
export function text(value: unknown): string {
  if (typeof value !== "string") {
    throw new Error("Invalid replay string");
  }
  return value;
}
export function integer(value: unknown): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value)) {
    throw new Error("Invalid replay integer");
  }
  return value;
}
export function nanoseconds(value: unknown): bigint {
  const parsed = text(value);
  if (!/^-?\d{1,30}$/.test(parsed)) {
    throw new Error("Invalid replay timestamp");
  }
  return BigInt(parsed);
}
export function parseReplayInput(value: unknown): ReplayInput {
  const input = row(value);
  if (input.version !== 1) {
    throw new Error("Unsupported replay version");
  }
  nanoseconds(input.start_ns);
  nanoseconds(input.duration_ns);
  const rows = (key: string) => {
    const items = input[key];
    if (!Array.isArray(items) || items.length > 100000) {
      throw new Error("Invalid replay section");
    }
    return items.map(row);
  };
  return {
    version: 1,
    partial: input.partial === true,
    start_ns: text(input.start_ns),
    duration_ns: text(input.duration_ns),
    snapshots: rows("snapshots").map((item) => ({
      generation: integer(item.generation),
      snapshot_kind: text(item.snapshot_kind),
      source_id: text(item.source_id),
      payload: row(item.payload),
    })),
    events: rows("events").map((item) => {
      nanoseconds(item.at_ns);
      return {
        generation: integer(item.generation),
        event_kind: text(item.event_kind),
        source_id: text(item.source_id),
        at_ns: text(item.at_ns),
        payload: row(item.payload),
      };
    }),
    intervals: rows("intervals").map((item) => {
      nanoseconds(item.start_ns);
      nanoseconds(item.end_ns);
      return {
        generation: integer(item.generation),
        start_ns: text(item.start_ns),
        end_ns: text(item.end_ns),
        loss_reason: typeof item.loss_reason === "string" ? item.loss_reason : null,
      };
    }),
    assets: rows("assets").map((item) => ({
      file_id: text(item.file_id),
      mime: typeof item.mime === "string" ? item.mime : null,
      archive_key: text(item.archive_key),
    })),
    actors: Object.fromEntries(
      Object.entries(row(input.actors)).map(([id, name]) => [id, text(name)]),
    ),
  };
}
