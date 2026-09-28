"use client";

import { useEffect, useRef, useState } from "react";

import { Button } from "@/components/ui/button";
import { roomCopy } from "@/features/rooms/room-copy";
import type { Locale } from "@/lib/i18n/config";

export function RemoteAudio({ stream, locale }: Readonly<{ stream: MediaStream; locale: Locale }>) {
  const element = useRef<HTMLAudioElement>(null);
  const [blocked, setBlocked] = useState(false);
  const text = roomCopy(locale);

  useEffect(() => {
    const audio = element.current;
    if (!audio) {
      return undefined;
    }
    audio.srcObject = stream;
    void audio.play().then(
      () => setBlocked(false),
      () => setBlocked(true),
    );
    return () => {
      audio.pause();
      if (audio.srcObject === stream) {
        audio.srcObject = null;
      }
    };
  }, [stream]);

  return (
    <>
      <audio autoPlay ref={element} />
      {blocked ? (
        <Button
          onClick={() =>
            void element.current?.play().then(
              () => setBlocked(false),
              () => setBlocked(true),
            )
          }
          size="sm"
          variant="outline"
        >
          {text.playRemoteAudio}
        </Button>
      ) : null}
    </>
  );
}
