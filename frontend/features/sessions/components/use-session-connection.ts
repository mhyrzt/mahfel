"use client";

import { useEffect, useState, type RefObject } from "react";

import {
  combineConnectionMetrics,
  emptyConnectionMetrics,
  readConnectionStats,
  type ConnectionMetrics,
  type PacketSample,
} from "@/lib/rtc/connection-stats";
import type { RtcTransport } from "@/lib/rtc/transport";
import type { SignalingStatus } from "@/lib/websocket/client";

export type MediaConnectionState =
  "idle" | "connecting" | "connected" | "disconnected" | "unavailable";

export interface SessionConnection {
  signaling: SignalingStatus;
  offline: boolean;
  media: MediaConnectionState;
  metrics: ConnectionMetrics;
}

export function useSessionConnection(
  signaling: SignalingStatus,
  active: boolean,
  sendRef: RefObject<RtcTransport | null>,
  receiveRef: RefObject<RtcTransport | null>,
): SessionConnection {
  const [offline, setOffline] = useState(false);
  const [sample, setSample] = useState<{
    media: MediaConnectionState;
    metrics: ConnectionMetrics;
  }>({ media: "idle", metrics: emptyConnectionMetrics });

  useEffect(() => {
    const update = () => setOffline(!navigator.onLine);
    update();
    window.addEventListener("online", update);
    window.addEventListener("offline", update);
    return () => {
      window.removeEventListener("online", update);
      window.removeEventListener("offline", update);
    };
  }, []);

  useEffect(() => {
    if (!active || signaling !== "connected" || offline) {
      return undefined;
    }
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    let previous = new Map<RtcTransport, Map<string, PacketSample>>();
    const refresh = async () => {
      const transports = [sendRef.current, receiveRef.current].filter(
        (transport): transport is RtcTransport => transport !== null,
      );
      const next = new Map<RtcTransport, Map<string, PacketSample>>();
      const results = await Promise.all(
        transports.map(async (transport) => {
          if (transport.closed || transport.connectionState !== "connected") {
            return null;
          }
          try {
            const report = await transport.getStats();
            if (transport.closed || transport.connectionState !== "connected") {
              return null;
            }
            const result = readConnectionStats(report, previous.get(transport) ?? new Map());
            next.set(transport, result.packets);
            return result.metrics;
          } catch {
            return null;
          }
        }),
      );
      if (disposed) {
        return;
      }
      previous = next;
      const unchanged = transports.every(
        (transport) => transport === sendRef.current || transport === receiveRef.current,
      );
      if (unchanged) {
        const failed = transports.some(
          (transport) =>
            transport.closed ||
            ["failed", "disconnected", "closed"].includes(transport.connectionState),
        );
        const connecting = transports.some(
          (transport) => transport.connectionState !== "connected",
        );
        const available = results.filter((result): result is ConnectionMetrics => result !== null);
        let media: MediaConnectionState = "idle";
        if (failed) {
          media = "disconnected";
        } else if (connecting) {
          media = "connecting";
        } else if (transports.length) {
          media = available.length === transports.length ? "connected" : "unavailable";
        }
        setSample({
          media,
          metrics:
            failed || connecting ? emptyConnectionMetrics : combineConnectionMetrics(available),
        });
      }
      timer = setTimeout(() => void refresh(), 3_000);
    };
    void refresh();
    return () => {
      disposed = true;
      clearTimeout(timer);
    };
  }, [active, offline, receiveRef, sendRef, signaling]);

  return {
    signaling,
    offline,
    media: active && signaling === "connected" && !offline ? sample.media : "idle",
    metrics:
      active && signaling === "connected" && !offline ? sample.metrics : emptyConnectionMetrics,
  };
}
