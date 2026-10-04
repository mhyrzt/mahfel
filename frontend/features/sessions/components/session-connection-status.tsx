"use client";

import { useId, useState } from "react";
import {
  ChevronDown,
  CircleCheck,
  LoaderCircle,
  SignalLow,
  SignalMedium,
  WifiOff,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "@/components/ui/hover-card";
import { connectionQuality } from "@/lib/rtc/connection-stats";
import { directionForLocale, type Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import { cn } from "@/lib/utils";

import type { SessionConnection } from "./use-session-connection";

const qualityFaces = { good: "🙂", fair: "😐", poor: "🙁" } as const;

export function SessionConnectionStatus({
  connection,
  locale,
}: Readonly<{ connection: SessionConnection; locale: Locale }>) {
  const [open, setOpen] = useState(false);
  const contentId = useId();
  const titleId = useId();
  const descriptionId = useId();
  const text = messagesFor(locale).sessionConnection;
  const status = connection.offline ? "offline" : connection.signaling;
  const mediaConnected = status === "connected" && connection.media === "connected";
  const quality = mediaConnected ? connectionQuality(connection.metrics) : "unknown";
  const qualityFace = quality === "unknown" ? null : qualityFaces[quality];
  let label = text.status[status];
  if (status === "connected") {
    if (connection.media === "disconnected" || connection.media === "connecting") {
      label = text.mediaStatus[connection.media];
    } else if (quality !== "unknown") {
      label = text.qualityLabel[quality];
    }
  }
  const disconnected =
    connection.offline ||
    ["closed", "unavailable", "unauthorized"].includes(status) ||
    connection.media === "disconnected";
  const pending =
    status === "connecting" || status === "reconnecting" || connection.media === "connecting";
  let Icon = CircleCheck;
  let tone = "text-success";
  let surface = "bg-success/10";
  if (disconnected) {
    Icon = WifiOff;
    tone = "text-destructive";
    surface = "bg-destructive/10";
  } else if (pending) {
    Icon = LoaderCircle;
    tone = "text-warning";
    surface = "bg-warning/10";
  } else if (quality === "poor") {
    Icon = SignalLow;
    tone = "text-destructive";
    surface = "bg-destructive/10";
  } else if (quality === "fair") {
    Icon = SignalMedium;
    tone = "text-warning";
    surface = "bg-warning/10";
  }
  const iconClass = cn("size-4 shrink-0", tone, pending && "motion-safe:animate-spin");
  const format = new Intl.NumberFormat(locale === "fa" ? "fa-IR" : "en", {
    maximumFractionDigits: 0,
  });
  const percentage = new Intl.NumberFormat(locale === "fa" ? "fa-IR" : "en", {
    style: "percent",
    maximumFractionDigits: 1,
  });
  const milliseconds = (value: number | null) =>
    value === null ? text.unavailable : `${format.format(value)} ms`;
  const metrics = mediaConnected
    ? connection.metrics
    : { latencyMs: null, jitterMs: null, packetLossPercent: null };

  return (
    <HoverCard open={open} onOpenChange={setOpen} openDelay={160} closeDelay={180}>
      <HoverCardTrigger asChild>
        <Button
          aria-label={`${text.title}: ${label}`}
          aria-expanded={open}
          aria-controls={open ? contentId : undefined}
          aria-describedby={open ? descriptionId : undefined}
          className="group h-10 min-w-10 shrink-0 gap-2 rounded-lg px-2.5 text-xs hover:bg-muted/70 data-[state=open]:bg-muted/70"
          onClick={() => setOpen(true)}
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              setOpen(false);
            }
          }}
          variant="ghost"
        >
          <Icon aria-hidden="true" className={iconClass} />
          {qualityFace ? (
            <span
              role="img"
              aria-label={text.qualityLabel[quality]}
              className="text-lg leading-none"
            >
              {qualityFace}
            </span>
          ) : (
            <span
              className={cn(
                "font-medium",
                tone,
                metrics.latencyMs === null ? "max-w-32 truncate" : "hidden sm:inline",
              )}
            >
              {label}
            </span>
          )}
          {metrics.latencyMs !== null ? (
            <span className="border-s border-border/70 ps-2 tabular-nums text-foreground">
              <bdi dir="ltr">{milliseconds(metrics.latencyMs)}</bdi>
            </span>
          ) : null}
          <ChevronDown
            aria-hidden="true"
            className={cn(
              "size-3 text-muted-foreground transition-transform motion-reduce:transition-none",
              open && "rotate-180",
            )}
          />
        </Button>
      </HoverCardTrigger>
      <HoverCardContent
        id={contentId}
        role="region"
        aria-labelledby={titleId}
        dir={directionForLocale(locale)}
        align="end"
        side="bottom"
        className="p-0 text-start"
      >
        <div className="flex items-center gap-3 px-4 pt-4 pb-3">
          <span
            className={cn("flex size-9 shrink-0 items-center justify-center rounded-lg", surface)}
          >
            <Icon aria-hidden="true" className={cn(iconClass, "size-5")} />
          </span>
          <div className="min-w-0">
            <h2 id={titleId} className="text-xs text-muted-foreground">
              {text.title}
            </h2>
            <p className={cn("mt-0.5 text-sm font-semibold", tone)}>
              {qualityFace ? (
                <span role="img" aria-label={label} className="text-xl leading-none">
                  {qualityFace}
                </span>
              ) : (
                label
              )}
            </p>
          </div>
        </div>
        <dl className="space-y-2 px-4 pb-3 text-xs">
          {[
            [text.signaling, text.status[status]],
            [text.media, text.mediaStatus[connection.media]],
            [text.quality, text.qualityLabel[quality]],
          ].map(([name, value]) => (
            <div className="flex items-start justify-between gap-4" key={name}>
              <dt className="text-muted-foreground">{name}</dt>
              <dd className="text-end font-medium">
                {name === text.quality && qualityFace ? (
                  <span role="img" aria-label={value} className="text-lg leading-none">
                    {qualityFace}
                  </span>
                ) : (
                  <bdi>{value}</bdi>
                )}
              </dd>
            </div>
          ))}
        </dl>
        <dl className="grid grid-cols-3 border-y border-border/70 bg-muted/30 py-3">
          {[
            [text.latency, milliseconds(metrics.latencyMs), metrics.latencyMs === null],
            [text.jitter, milliseconds(metrics.jitterMs), metrics.jitterMs === null],
            [
              text.packetLoss,
              metrics.packetLossPercent === null
                ? text.unavailable
                : percentage.format(metrics.packetLossPercent / 100),
              metrics.packetLossPercent === null,
            ],
          ].map(([name, value, unavailable]) => (
            <div
              className="min-w-0 border-s border-border/70 px-3 first:border-s-0"
              key={String(name)}
            >
              <dt className="text-xs leading-4 text-muted-foreground">{name}</dt>
              <dd
                className={cn(
                  "mt-1.5 font-semibold tabular-nums",
                  unavailable ? "text-xs text-muted-foreground" : "text-sm",
                )}
              >
                <bdi dir={unavailable ? "auto" : "ltr"}>{value}</bdi>
              </dd>
            </div>
          ))}
        </dl>
        <div
          id={descriptionId}
          className="space-y-2 px-4 py-3 text-xs leading-relaxed text-muted-foreground"
        >
          {connection.media === "idle" ? <p>{text.idleHint}</p> : null}
          <p>{text.estimate}</p>
        </div>
      </HoverCardContent>
    </HoverCard>
  );
}
