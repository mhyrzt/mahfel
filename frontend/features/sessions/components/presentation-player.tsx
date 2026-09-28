"use client";

import "@vidstack/react/player/styles/default/theme.css";
import "@vidstack/react/player/styles/default/layouts/video.css";

import { MediaPlayer, MediaProvider, type MediaPlayerInstance } from "@vidstack/react";
import { DefaultVideoLayout, defaultLayoutIcons } from "@vidstack/react/player/layouts/default";
import { Volume2, VolumeX } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { Button } from "@/components/ui/button";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";

import { shouldCorrectDrift, type PlaybackState } from "./presentation-playback";

const HEARTBEAT_MS = 4000;

export function PresentationPlayer({
  url,
  canPresent,
  playback,
  locale,
  onPlayback,
}: Readonly<{
  url: string;
  canPresent: boolean;
  playback: PlaybackState | null;
  locale: Locale;
  onPlayback: (state: PlaybackState) => void;
}>) {
  const text = messagesFor(locale).collaboration;
  const player = useRef<MediaPlayerInstance>(null);
  const applyingRemote = useRef(false);
  const lastHeartbeat = useRef(0);
  const [ready, setReady] = useState(false);
  const [blocked, setBlocked] = useState(false);
  const [muted, setMuted] = useState(false);

  const emit = useCallback(
    (paused: boolean) => {
      const instance = player.current;
      if (!instance || applyingRemote.current || !canPresent) {
        return;
      }
      onPlayback({
        positionMs: Math.max(0, Math.round(instance.currentTime * 1000)),
        paused,
        rate: instance.playbackRate,
      });
    },
    [canPresent, onPlayback],
  );

  useEffect(() => {
    const instance = player.current;
    if (!instance || !playback || !ready) {
      return;
    }
    applyingRemote.current = true;
    if (playback.paused !== instance.paused) {
      if (playback.paused) {
        instance.pause();
      } else {
        void instance.play().catch(() => setBlocked(true));
      }
    }
    if (shouldCorrectDrift(instance.currentTime * 1000, playback.positionMs)) {
      instance.currentTime = playback.positionMs / 1000;
    }
    if (Math.abs(instance.playbackRate - playback.rate) > 0.001) {
      instance.playbackRate = playback.rate;
    }
    window.setTimeout(() => {
      applyingRemote.current = false;
    }, 0);
  }, [playback, ready]);

  function toggleMute() {
    const instance = player.current;
    if (!instance) {
      return;
    }
    instance.muted = !instance.muted;
    setMuted(instance.muted);
  }

  return (
    <div className="relative h-full min-h-0 w-full" data-testid="presentation-player">
      <MediaPlayer
        className="h-full w-full"
        onCanPlay={() => setReady(true)}
        onPause={() => emit(true)}
        onPlay={() => emit(false)}
        onRateChange={() => emit(player.current?.paused ?? true)}
        onSeeked={() => emit(player.current?.paused ?? true)}
        onTimeUpdate={() => {
          if (applyingRemote.current || !canPresent) {
            return;
          }
          if (player.current?.paused ?? true) {
            return;
          }
          const now = Date.now();
          if (now - lastHeartbeat.current < HEARTBEAT_MS) {
            return;
          }
          lastHeartbeat.current = now;
          emit(false);
        }}
        playsInline
        ref={player}
        src={url}
        streamType="on-demand"
        title={text.presentation}
      >
        <MediaProvider />
        {canPresent ? <DefaultVideoLayout icons={defaultLayoutIcons} /> : null}
      </MediaPlayer>
      {!canPresent ? (
        <>
          <Button
            aria-label={muted ? text.unmute : text.mute}
            className="absolute end-2 top-2 z-10 size-9"
            onClick={toggleMute}
            size="icon"
            type="button"
            variant="secondary"
          >
            {muted ? (
              <VolumeX aria-hidden="true" className="size-4" />
            ) : (
              <Volume2 aria-hidden="true" className="size-4" />
            )}
          </Button>
          {blocked ? (
            <div className="absolute inset-0 z-20 grid place-items-center bg-black/50">
              <Button
                onClick={() => {
                  void player.current
                    ?.play()
                    .then(() => setBlocked(false))
                    .catch(() => undefined);
                }}
                type="button"
              >
                {text.watchToStart}
              </Button>
            </div>
          ) : null}
        </>
      ) : null}
    </div>
  );
}
