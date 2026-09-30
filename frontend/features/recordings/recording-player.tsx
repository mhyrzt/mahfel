"use client";

import { useEffect, useRef, useState } from "react";
import { Pause, Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import type { RecordingPlayback } from "./export-types";
import { loadReplayInput } from "./replay/manifest";
import type { ReplayInput, VideoTrack } from "./replay/types";
import "./renderer/renderer.css";

interface WorkspaceRenderer {
  initialize(input: ReplayInput): Promise<void>;
  render(timestamp: string, tracks: VideoTrack[], width: number, height: number): Promise<unknown>;
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
  const input = useRef<ReplayInput | null>(null);
  const offset = useRef(0);
  const startedAt = useRef(0);
  const busy = useRef(false);
  const [failed, setFailed] = useState(false);
  const [ready, setReady] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [position, setPosition] = useState(0);
  const [duration, setDuration] = useState(0);

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
        input.current = replay;
        await instance.render("0", [], 960, 540);
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
      renderer.current = null;
      input.current = null;
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
          .render(timestamp, [], 960, 540)
          .catch(() => setFailed(true))
          .finally(() => {
            busy.current = false;
          });
      }
      if (elapsed >= total) {
        offset.current = total;
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
      return;
    }
    if (position >= duration) {
      offset.current = 0;
      setPosition(0);
    }
    startedAt.current = performance.now();
    setPlaying(true);
  };

  const seek = (milliseconds: number) => {
    offset.current = milliseconds;
    setPosition(milliseconds);
    startedAt.current = performance.now();
    const instance = renderer.current;
    if (instance) {
      const timestamp = (BigInt(Math.round(milliseconds)) * BigInt(1_000_000)).toString();
      void instance.render(timestamp, [], 960, 540).catch(() => setFailed(true));
    }
  };

  return (
    <div className="space-y-3" dir={locale === "fa" ? "rtl" : "ltr"}>
      {failed ? (
        <p role="alert" className="text-sm text-destructive">
          {text.unavailable}
        </p>
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
        <div ref={container} className="recording-scene relative" />
      </div>
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
        <input
          aria-label={text.seek}
          className="h-1.5 w-full accent-primary"
          disabled={!ready}
          max={Math.max(0, duration)}
          min={0}
          onChange={(event) => seek(Number(event.target.value))}
          step={SEEK_STEP_MS}
          type="range"
          value={Math.min(position, Math.max(0, duration))}
        />
        <span aria-label={text.position} className="shrink-0 text-xs text-muted-foreground">
          {formatTime(position)} / {formatTime(duration)}
        </span>
      </div>
    </div>
  );
}
