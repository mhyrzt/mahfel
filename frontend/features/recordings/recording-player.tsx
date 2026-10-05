"use client";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { Checkbox } from "@/components/ui/checkbox";
import { FieldSet, FieldLegend, FieldLabel } from "@/components/ui/field";
import { Slider } from "@/components/ui/slider";
import { useEffect, useRef, useState } from "react";
import { Pause, Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import type { RecordingPlayback } from "./export-types";
import { MediaPlayback, videoTracks, type MediaSegment } from "./media-playback";
import { loadReplayInput } from "./replay/manifest";
import type { ReplayInput, SceneV1, VideoTrack } from "./replay/types";
import "./renderer/renderer.css";

interface WorkspaceRenderer {
  mediaRoot: HTMLElement;
  initialize(input: ReplayInput): Promise<void>;
  render(
    timestamp: string,
    tracks: VideoTrack[],
    width: number,
    height: number,
  ): Promise<{ scene: SceneV1 }>;
  destroy(): void;
}

const FRAME_INTERVAL_MS = 120;
const SEEK_STEP_MS = 1000;

function formatTime(milliseconds: number): string {
  const total = Math.max(0, Math.floor(milliseconds / 1000));
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function RecordingPlayer({
  sessionId,
  recordingId,
  locale,
}: Readonly<{ sessionId: string; recordingId: string; locale: Locale }>) {
  const text = messagesFor(locale).recordingPlayback;
  const container = useRef<HTMLDivElement>(null);
  const renderer = useRef<WorkspaceRenderer | null>(null);
  const media = useRef<MediaPlayback | null>(null);
  const scene = useRef<SceneV1 | null>(null);
  const segments = useRef<MediaSegment[]>([]);
  const offset = useRef(0);
  const startedAt = useRef(0);
  const busy = useRef(false);
  const [failed, setFailed] = useState(false);
  const [ready, setReady] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [position, setPosition] = useState(0);
  const [duration, setDuration] = useState(0);
  const [audioTracks, setAudioTracks] = useState<MediaSegment[]>([]);
  const [actorNames, setActorNames] = useState<Record<string, string>>({});
  const [muted, setMuted] = useState<Set<string>>(new Set());

  useEffect(() => {
    const root = container.current;
    const viewport = root?.parentElement;
    if (!root || !viewport) {
      return undefined;
    }
    const resize = () => {
      root.style.transform = `scale(${Math.min(1, viewport.clientWidth / 960)})`;
    };
    const observer = new ResizeObserver(resize);
    observer.observe(viewport);
    resize();
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const path = `/v1/sessions/${encodeURIComponent(sessionId)}/recordings/${encodeURIComponent(recordingId)}/playback`;
    const abort = new AbortController();
    let active = true;
    let instance: WorkspaceRenderer | null = null;
    void (async () => {
      try {
        const { data } = await browserApiClient.get<RecordingPlayback>(path, {
          signal: abort.signal,
        });
        const replay = await loadReplayInput(data, abort.signal);
        if (!active || !container.current) {
          return;
        }
        const { createWorkspaceRenderer } = await import("./renderer/workspace");
        if (!active || !container.current) {
          return;
        }
        instance = createWorkspaceRenderer(
          container.current,
          "/recording-renderer",
          (id) => data.assets[id] ?? "",
        );
        renderer.current = instance;
        await instance.initialize(replay);
        if (!active) {
          return;
        }
        setActorNames(replay.actors);
        segments.current = data.media;
        const controller = new MediaPlayback(
          instance.mediaRoot,
          data.media,
          async (startMs) => {
            const { data: window } = await browserApiClient.get<{ urls: Record<string, string> }>(
              `${path}/media-window?start_ms=${startMs}`,
              { signal: abort.signal },
            );
            return window.urls;
          },
          () => setFailed(true),
        );
        media.current = controller;
        setAudioTracks(controller.audioTracks());
        const frame = await instance.render("0", videoTracks(data.media, 0), 960, 540);
        scene.current = frame.scene;
        controller.update(0, false, frame.scene);
        setDuration(Number(BigInt(replay.duration_ns) / BigInt(1_000_000)));
        setFailed(false);
        setReady(true);
      } catch {
        if (active) {
          setFailed(true);
          setReady(false);
        }
      }
    })();
    return () => {
      active = false;
      abort.abort();
      instance?.destroy();
      media.current?.destroy();
      media.current = null;
      scene.current = null;
      segments.current = [];
      renderer.current = null;
    };
  }, [sessionId, recordingId]);

  useEffect(() => {
    if (!playing || !ready) {
      return undefined;
    }
    let frame = 0;
    const total = duration;
    const tick = () => {
      const elapsed = Math.min(total, offset.current + (performance.now() - startedAt.current));
      setPosition(elapsed);
      const instance = renderer.current;
      if (instance && !busy.current) {
        busy.current = true;
        const timestamp = (BigInt(Math.round(elapsed)) * BigInt(1_000_000)).toString();
        void instance
          .render(timestamp, videoTracks(segments.current, elapsed), 960, 540)
          .then((frame) => {
            scene.current = frame.scene;
            media.current?.update(elapsed, true, frame.scene);
          })
          .catch(() => setFailed(true))
          .finally(() => {
            busy.current = false;
          });
      }
      if (elapsed >= total) {
        offset.current = total;
        if (scene.current) {
          media.current?.update(total, false, scene.current);
        }
        setPlaying(false);
        return;
      }
      frame = window.setTimeout(tick, FRAME_INTERVAL_MS);
    };
    frame = window.setTimeout(tick, 0);
    return () => window.clearTimeout(frame);
  }, [playing, ready, duration]);

  const toggle = () => {
    if (playing) {
      offset.current += performance.now() - startedAt.current;
      setPlaying(false);
      if (scene.current) {
        media.current?.update(offset.current, false, scene.current);
      }
      return;
    }
    if (position >= duration) {
      offset.current = 0;
      setPosition(0);
    }
    startedAt.current = performance.now();
    setPlaying(true);
    if (scene.current) {
      media.current?.update(offset.current, true, scene.current);
    }
  };

  const seek = (milliseconds: number) => {
    offset.current = milliseconds;
    setPosition(milliseconds);
    startedAt.current = performance.now();
    const instance = renderer.current;
    if (instance) {
      const timestamp = (BigInt(Math.round(milliseconds)) * BigInt(1_000_000)).toString();
      void instance
        .render(timestamp, videoTracks(segments.current, milliseconds), 960, 540)
        .then((frame) => {
          scene.current = frame.scene;
          media.current?.update(milliseconds, playing, frame.scene);
        })
        .catch(() => setFailed(true));
    }
  };

  return (
    <div className="space-y-3" dir={locale === "fa" ? "rtl" : "ltr"}>
      {failed ? (
        <Alert variant="destructive" className="text-sm text-destructive">
          <AlertDescription>{text.unavailable}</AlertDescription>
        </Alert>
      ) : null}
      {!ready && !failed ? (
        <p role="status" className="text-sm text-muted-foreground">
          {text.loading}
        </p>
      ) : null}
      <div
        className="relative w-[960px] max-w-full overflow-hidden rounded-xl bg-slate-950"
        style={{ aspectRatio: "16/9" }}
      >
        <div ref={container} className="recording-scene relative origin-top-left" />
      </div>
      {audioTracks.length > 0 ? (
        <FieldSet className="flex flex-row flex-wrap gap-3 text-sm">
          <FieldLegend className="mb-1 text-muted-foreground">{text.audio}</FieldLegend>
          {audioTracks.map((track) => (
            <FieldLabel key={track.track_id} className="flex items-center gap-2">
              <Checkbox
                checked={!muted.has(track.track_id)}
                onCheckedChange={(checked) => {
                  const enabled = checked === true;
                  media.current?.setMuted(track.track_id, !enabled);
                  setMuted((previous) => {
                    const next = new Set(previous);
                    if (enabled) {
                      next.delete(track.track_id);
                    } else {
                      next.add(track.track_id);
                    }
                    return next;
                  });
                }}
              />
              {actorNames[track.actor_id] ?? track.source}
            </FieldLabel>
          ))}
        </FieldSet>
      ) : null}
      <div className="flex items-center gap-3">
        <Button
          aria-label={playing ? text.pause : text.play}
          disabled={!ready}
          onClick={toggle}
          size="icon"
          variant="outline"
        >
          {playing ? (
            <Pause aria-hidden="true" className="size-4" />
          ) : (
            <Play aria-hidden="true" className="size-4" />
          )}
        </Button>
        <Slider
          dir="ltr"
          aria-valuetext={formatTime(position)}
          aria-label={text.seek}
          className="h-6 w-full"
          disabled={!ready || duration <= 0}
          max={Math.max(SEEK_STEP_MS, duration)}
          min={0}
          onValueChange={([next]) => {
            if (next !== undefined) {
              seek(next);
            }
          }}
          step={SEEK_STEP_MS}
          value={[Math.min(position, Math.max(0, duration))]}
        />
        <span aria-label={text.position} className="shrink-0 text-xs text-muted-foreground">
          {formatTime(position)} / {formatTime(duration)}
        </span>
      </div>
    </div>
  );
}
