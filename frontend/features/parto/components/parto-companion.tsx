"use client";

import { useEffect, useRef, useState } from "react";
import { AudioLines, MessageCircle } from "lucide-react";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import { Parto } from "./parto";

export function PartoCompanion({ locale }: Readonly<{ locale: Locale }>) {
  const copy = messagesFor(locale).home;
  const container = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState(320);
  const [talking, setTalking] = useState(true);
  useEffect(() => {
    const timer = window.setTimeout(() => setTalking(false), 6500);
    return () => window.clearTimeout(timer);
  }, []);
  useEffect(() => {
    const element = container.current;
    if (!element) {
      return undefined;
    }
    const observer = new ResizeObserver(([entry]) => {
      if (entry.contentRect.width > 0) {
        setSize(Math.min(380, Math.floor(entry.contentRect.width)));
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  return (
    <div
      ref={container}
      className="relative mx-auto w-full max-w-[380px] pb-1"
      dir={locale === "fa" ? "rtl" : "ltr"}
    >
      <div
        data-parto-bubble
        className="relative z-10 mx-auto w-[92%] rounded-2xl border border-border/80 bg-card px-5 py-4 sm:px-6 sm:py-5 text-card-foreground shadow-[0_8px_28px_-16px_color-mix(in_srgb,var(--primary)_25%,transparent)]"
      >
        <div className="mb-2 flex items-center gap-2 text-xs font-medium text-accent-foreground">
          {talking ? (
            <AudioLines className="size-4" aria-hidden="true" />
          ) : (
            <MessageCircle className="size-4" aria-hidden="true" />
          )}
          <span className="font-semibold">{copy.partoName}</span>
          <span className="text-muted-foreground" role="status">
            {talking ? copy.partoTalking : copy.partoReady}
          </span>
        </div>
        <p className="text-sm leading-7">{copy.partoGreeting}</p>
        <div
          aria-hidden="true"
          className="absolute -bottom-2 start-[52%] size-4 rotate-45 border-b border-e border-border/80 bg-card"
        />
      </div>
      <div className="-mt-10 flex aspect-square items-center justify-center">
        <Parto
          size={size}
          design="faithful"
          activity={talking ? "speaking" : "idle"}
          emotion="curious"
          audioLevel={talking ? 0.6 : 0}
          energy={0.35}
          happiness={0.75}
          interactive
          followCursor
          label={copy.partoName}
          interactionLabel={copy.partoInteract}
          errorLabel={copy.partoUnavailable}
        />
      </div>
    </div>
  );
}
