import type { SceneV1, VideoTrack } from "./replay/types";

export interface MediaSegment {
  track_id: string;
  actor_id: string;
  source: string;
  audio: boolean;
  start_ns: string;
  end_ns: string;
  object_key: string;
}

export function activeSegments(segments: MediaSegment[], positionMs: number): MediaSegment[] {
  const at = BigInt(Math.round(positionMs)) * BigInt(1_000_000);
  const selected = new Map<string, MediaSegment>();
  for (const segment of segments) {
    if (
      BigInt(segment.start_ns) <= at &&
      at < BigInt(segment.end_ns) &&
      (!selected.has(segment.track_id) ||
        BigInt(selected.get(segment.track_id)?.start_ns ?? "0") < BigInt(segment.start_ns))
    ) {
      selected.set(segment.track_id, segment);
    }
  }
  return [...selected.values()];
}

export function videoTracks(segments: MediaSegment[], positionMs: number): VideoTrack[] {
  return activeSegments(segments, positionMs)
    .filter((segment) => segment.source === "camera" || segment.source === "screen")
    .map((segment) => ({
      id: segment.track_id,
      actorId: segment.actor_id,
      source: segment.source as "camera" | "screen",
      started_ns: segment.start_ns,
    }));
}

interface PlayingElement {
  element: HTMLMediaElement;
  key: string;
}

export class MediaPlayback {
  private readonly root: HTMLElement;
  private readonly segments: MediaSegment[];
  private readonly getWindow: (startMs: number) => Promise<Record<string, string>>;
  private readonly onError: () => void;
  private readonly elements = new Map<string, PlayingElement>();
  private readonly urls = new Map<string, string>();
  private readonly muted = new Set<string>();
  private windowStart = -1;
  private windowFetchedAt = 0;
  private windowFailedAt = 0;
  private loading: Promise<void> | null = null;
  private destroyed = false;
  private last: { positionMs: number; playing: boolean; scene: SceneV1 } | null = null;

  constructor(
    root: HTMLElement,
    segments: MediaSegment[],
    getWindow: (startMs: number) => Promise<Record<string, string>>,
    onError: () => void,
  ) {
    this.root = root;
    this.segments = segments;
    this.getWindow = getWindow;
    this.onError = onError;
  }

  audioTracks(): MediaSegment[] {
    return [
      ...new Map(
        this.segments
          .filter((segment) => segment.audio)
          .map((segment) => [segment.track_id, segment]),
      ).values(),
    ];
  }

  setMuted(track: string, muted: boolean): void {
    if (muted) {
      this.muted.add(track);
    } else {
      this.muted.delete(track);
    }
    const element = this.elements.get(track)?.element;
    if (element) {
      element.muted = muted;
    }
  }

  private ensureWindow(positionMs: number): void {
    const start = Math.floor(positionMs / 20_000) * 20_000;
    if (
      this.loading ||
      Date.now() - this.windowFailedAt < 5000 ||
      (start === this.windowStart && Date.now() - this.windowFetchedAt < 240_000)
    ) {
      return;
    }
    this.loading = this.getWindow(start)
      .then((urls) => {
        if (this.destroyed) {
          return;
        }
        this.urls.clear();
        for (const [key, url] of Object.entries(urls)) {
          this.urls.set(key, url);
        }
        this.windowStart = start;
        this.windowFetchedAt = Date.now();
        if (this.last) {
          this.update(this.last.positionMs, this.last.playing, this.last.scene);
        }
      })
      .catch(() => {
        if (this.destroyed) {
          return;
        }
        this.windowFailedAt = Date.now();
        this.onError();
      })
      .finally(() => {
        this.loading = null;
      });
  }

  update(positionMs: number, playing: boolean, scene: SceneV1): void {
    if (this.destroyed) {
      return;
    }
    this.last = { positionMs, playing, scene };
    this.ensureWindow(positionMs);
    const active = activeSegments(this.segments, positionMs);
    const visible = new Set(active.map((segment) => segment.track_id));
    for (const [track, entry] of this.elements) {
      if (!visible.has(track)) {
        entry.element.pause();
        entry.element.removeAttribute("src");
        entry.element.load();
        entry.element.remove();
        this.elements.delete(track);
      }
    }
    for (const segment of active) {
      const url = this.urls.get(segment.object_key);
      if (!url) {
        continue;
      }
      let entry = this.elements.get(segment.track_id);
      if (!entry || entry.key !== segment.object_key) {
        entry?.element.pause();
        entry?.element.remove();
        const element = document.createElement(segment.audio ? "audio" : "video");
        element.preload = "auto";
        element.crossOrigin = "anonymous";
        element.src = url;
        element.muted = !segment.audio || this.muted.has(segment.track_id);
        element.addEventListener("error", this.onError);
        if (!segment.audio) {
          element.setAttribute("playsinline", "");
          element.style.position = "absolute";
          element.style.objectFit = "cover";
          element.style.pointerEvents = "none";
        } else {
          element.hidden = true;
        }
        this.root.append(element);
        entry = { element, key: segment.object_key };
        this.elements.set(segment.track_id, entry);
      }
      if (!segment.audio) {
        const layer = scene.layers.find((item) => item.source === segment.track_id);
        entry.element.style.display = layer ? "block" : "none";
        if (layer) {
          entry.element.style.left = `${(layer.x / scene.canvas.width) * 100}%`;
          entry.element.style.top = `${(layer.y / scene.canvas.height) * 100}%`;
          entry.element.style.width = `${(layer.width / scene.canvas.width) * 100}%`;
          entry.element.style.height = `${(layer.height / scene.canvas.height) * 100}%`;
          entry.element.style.objectFit = layer.fit;
          entry.element.style.zIndex = String(layer.z);
        }
      }
      const localSeconds =
        (positionMs - Number(BigInt(segment.start_ns) / BigInt(1_000_000))) / 1000;
      const base = entry.element.seekable.length > 0 ? entry.element.seekable.start(0) : 0;
      const targetSeconds = base + Math.max(0, localSeconds);
      if (
        entry.element.readyState >= HTMLMediaElement.HAVE_METADATA &&
        Math.abs(entry.element.currentTime - targetSeconds) > 0.2
      ) {
        entry.element.currentTime = targetSeconds;
      }
      if (playing) {
        if (entry.element.paused) {
          void entry.element.play().catch((error: unknown) => {
            if (error instanceof DOMException && error.name === "AbortError") {
              return;
            }
            this.onError();
          });
        }
      } else {
        entry.element.pause();
      }
    }
  }

  destroy(): void {
    this.destroyed = true;
    for (const { element } of this.elements.values()) {
      element.pause();
      element.removeAttribute("src");
      element.load();
      element.remove();
    }
    this.elements.clear();
  }
}
