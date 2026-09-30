import { parseReplayInput, row, text, type ReplayInput } from "./types";

export interface PlaybackDescriptor {
  manifest_key: string;
  urls: Record<string, string>;
  assets: Record<string, string>;
}

const MAX_PAGES = 4096;

async function fetchJson(url: string, signal?: AbortSignal): Promise<unknown> {
  const response = await fetch(url, { signal, credentials: "omit" });
  if (!response.ok) {
    throw new Error("Recording archive unavailable");
  }
  return response.json();
}

function epochNanoseconds(value: string): bigint {
  const match = /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})(?:\.(\d+))?(Z|[+-]\d{2}:\d{2})$/.exec(
    value,
  );
  if (!match) {
    throw new Error("Invalid recording timestamp");
  }
  const millis = Date.parse(`${match[1]}${match[3]}`);
  const fraction = (match[2] ?? "").padEnd(9, "0").slice(0, 9);
  return BigInt(millis) * BigInt(1_000_000) + BigInt(fraction);
}

export async function loadReplayInput(
  descriptor: PlaybackDescriptor,
  signal?: AbortSignal,
): Promise<ReplayInput> {
  const manifestUrl = descriptor.urls[descriptor.manifest_key];
  if (!manifestUrl) {
    throw new Error("Replay manifest unavailable");
  }
  const manifest = row(await fetchJson(manifestUrl, signal));
  if (manifest.version !== 1) {
    throw new Error("Unsupported replay version");
  }
  const recording = row(manifest.recording);
  const started = epochNanoseconds(text(recording.started_at));
  const stopped = epochNanoseconds(text(recording.stopped_at));
  const origin = BigInt(text(recording.session_origin_ns));
  const sections = row(manifest.sections);
  const pages = (kind: string): unknown[] => {
    const value = sections[kind];
    if (value === undefined) {
      return [];
    }
    if (!Array.isArray(value) || value.length > MAX_PAGES) {
      throw new Error("Invalid replay section");
    }
    return value;
  };
  const entries = async (kind: string): Promise<Record<string, unknown>[]> => {
    const rows: Record<string, unknown>[] = [];
    for (const page of pages(kind)) {
      const objectKey = text(row(page).object_key);
      const url = descriptor.urls[objectKey];
      if (!url) {
        throw new Error("Replay page unavailable");
      }
      const value = await fetchJson(url, signal);
      if (!Array.isArray(value) || value.length > 100000) {
        throw new Error("Invalid replay page");
      }
      rows.push(...value.map((item) => row(item)));
    }
    return rows;
  };
  const assets = (await entries("assets")).map((asset) => {
    const fileId = text(asset.file_id);
    return {
      file_id: fileId,
      mime: typeof asset.mime === "string" ? asset.mime : null,
      archive_key: descriptor.assets[fileId] ?? text(asset.archive_key),
    };
  });
  const actors = Object.fromEntries(
    (await entries("actors")).map((actor) => [text(actor.actor_id), text(actor.display_name)]),
  );
  const intervals = Array.isArray(manifest.intervals) ? manifest.intervals : [];
  return parseReplayInput({
    version: 1,
    partial: manifest.partial === true,
    start_ns: (started - origin).toString(),
    duration_ns: (stopped - started).toString(),
    snapshots: await entries("snapshots"),
    events: await entries("events"),
    intervals,
    assets,
    actors,
  });
}
