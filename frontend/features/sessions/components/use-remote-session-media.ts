"use client";

import { useCallback, useEffect, useRef, useState, type RefObject } from "react";
import type { Device } from "mediasoup-client";
import type { Consumer } from "mediasoup-client/types";

import { roomCopy } from "@/features/rooms/room-copy";
import { createDevice } from "@/lib/rtc/device";
import { createRecvTransport, type RtcTransport } from "@/lib/rtc/transport";
import type { Locale } from "@/lib/i18n/config";
import type { SignalingClient } from "@/lib/websocket/client";
import type { ServerMessage } from "@/lib/websocket/generated/messages";

type Producer = Extract<ServerMessage, { type: "producer_snapshot" }>["producers"][number];
type Source = "microphone" | "camera" | "screen";
type ValidProducer = Producer &
  ({ kind: "audio"; source: "microphone" } | { kind: "video"; source: "camera" | "screen" });
type RemoteSource = { actorId: string; source: Source; consumer: Consumer; stream: MediaStream };

export type RemoteMedia = Record<string, Partial<Record<Source, MediaStream>>>;

const POLL_INTERVAL_MS = 3_000;
const MAX_PRODUCERS = 256;

function validProducer(entry: Producer): entry is ValidProducer {
  return (
    (entry.kind === "audio" && entry.source === "microphone") ||
    (entry.kind === "video" && (entry.source === "camera" || entry.source === "screen"))
  );
}

export function useRemoteSessionMedia(
  clientRef: RefObject<SignalingClient | null>,
  roomId: string,
  actorId: string | undefined,
  active: boolean,
  locale: Locale,
  onFailure?: () => void,
) {
  const [streams, setStreams] = useState<RemoteMedia>({});
  const [error, setError] = useState<string | null>(null);
  const transportRef = useRef<RtcTransport | null>(null);
  const transportPromiseRef = useRef<Promise<{ device: Device; transport: RtcTransport }> | null>(
    null,
  );
  const deviceRef = useRef<Device | null>(null);
  const consumersRef = useRef(new Map<string, RemoteSource>());
  const generationRef = useRef(0);
  const failureText = roomCopy(locale).mediaUnavailable;

  const request = useCallback(
    async (message: Parameters<SignalingClient["request"]>[0]) => {
      const client = clientRef.current;
      if (!client) {
        throw new Error("Session signaling is unavailable");
      }
      return client.request(message);
    },
    [clientRef],
  );

  const publishStreams = useCallback(() => {
    const next: RemoteMedia = {};
    for (const { actorId: owner, source, stream } of consumersRef.current.values()) {
      next[owner] ??= {};
      next[owner][source] = stream;
    }
    setStreams(next);
  }, []);

  const ensureTransport = useCallback(
    async (generation: number) => {
      if (transportRef.current && !transportRef.current.closed && deviceRef.current) {
        return { device: deviceRef.current, transport: transportRef.current };
      }
      if (transportPromiseRef.current) {
        return transportPromiseRef.current;
      }
      const pending = (async () => {
        const response = await request({ type: "create_transport", room_id: roomId });
        if (response.type !== "transport_created" || !response.rtp_capabilities) {
          throw new Error("Media transport parameters are unavailable");
        }
        const device = await createDevice(JSON.parse(response.rtp_capabilities));
        const transport = createRecvTransport(
          device,
          {
            id: response.transport_id,
            iceParameters: JSON.parse(response.ice_parameters),
            iceCandidates: JSON.parse(response.ice_candidates),
            dtlsParameters: JSON.parse(response.dtls_parameters),
          },
          async (dtlsParameters) => {
            const connected = await request({
              type: "connect_transport",
              room_id: roomId,
              transport_id: response.transport_id,
              dtls_parameters: JSON.stringify(dtlsParameters),
            });
            if (connected.type !== "transport_connected") {
              throw new Error("Media receive transport connection failed");
            }
          },
        );
        if (generation !== generationRef.current) {
          transport.close();
          throw new Error("Session media changed");
        }
        transport.on("connectionstatechange", (state) => {
          if (generation !== generationRef.current) {
            return;
          }
          if (state === "failed" || state === "disconnected") {
            setError(failureText);
            if (state === "failed") {
              onFailure?.();
            }
          }
        });
        deviceRef.current = device;
        transportRef.current = transport;
        return { device, transport };
      })();
      transportPromiseRef.current = pending;
      try {
        return await pending;
      } finally {
        if (transportPromiseRef.current === pending) {
          transportPromiseRef.current = null;
        }
      }
    },
    [failureText, onFailure, request, roomId],
  );

  const consume = useCallback(
    async (entry: Producer, generation: number) => {
      if (!validProducer(entry) || consumersRef.current.has(entry.producer_id)) {
        return;
      }
      const { device, transport } = await ensureTransport(generation);
      const response = await request({
        type: "consume",
        room_id: roomId,
        transport_id: transport.id,
        producer_id: entry.producer_id,
        rtp_capabilities: JSON.stringify(device.rtpCapabilities),
      });
      if (response.type !== "consumed") {
        throw new Error("Media consumption failed");
      }
      const consumer = await transport.consume({
        id: response.consumer_id,
        producerId: entry.producer_id,
        kind: entry.kind,
        rtpParameters: JSON.parse(response.rtp_parameters),
      });
      if (generation !== generationRef.current) {
        consumer.close();
        return;
      }
      const stream = new MediaStream([consumer.track]);
      consumersRef.current.set(entry.producer_id, {
        actorId: entry.actor_id,
        source: entry.source,
        consumer,
        stream,
      });
      publishStreams();
      if (response.paused) {
        try {
          const resumed = await request({
            type: "resume_consumer",
            room_id: roomId,
            consumer_id: response.consumer_id,
          });
          if (resumed.type !== "consumer_resumed") {
            throw new Error("Media consumer could not resume");
          }
        } catch (failure) {
          consumer.close();
          consumersRef.current.delete(entry.producer_id);
          publishStreams();
          throw failure;
        }
      }
    },
    [ensureTransport, publishStreams, request, roomId],
  );

  useEffect(() => {
    if (!active) {
      return undefined;
    }
    const generation = ++generationRef.current;
    let polling = false;
    const refresh = async () => {
      if (polling) {
        return;
      }
      polling = true;
      try {
        const response = await request({ type: "list_producers", room_id: roomId });
        if (response.type !== "producer_snapshot" || response.producers.length > MAX_PRODUCERS) {
          throw new Error("Media producer snapshot unavailable");
        }
        if (generation !== generationRef.current) {
          return;
        }
        const remote = response.producers.filter((entry) => entry.actor_id !== actorId);
        const currentIds = new Set(remote.map((entry) => entry.producer_id));
        let removed = false;
        for (const [id, source] of consumersRef.current) {
          if (!currentIds.has(id)) {
            source.consumer.close();
            consumersRef.current.delete(id);
            removed = true;
          }
        }
        if (removed) {
          publishStreams();
        }
        let failed = false;
        for (const entry of remote) {
          if (generation !== generationRef.current) {
            return;
          }
          try {
            await consume(entry, generation);
          } catch {
            failed = true;
          }
        }
        if (generation === generationRef.current) {
          setError(failed ? failureText : null);
        }
      } catch {
        if (generation === generationRef.current) {
          setError(failureText);
        }
      } finally {
        polling = false;
      }
    };
    void refresh();
    const timer = window.setInterval(() => void refresh(), POLL_INTERVAL_MS);
    const consumers = consumersRef.current;
    return () => {
      generationRef.current += 1;
      window.clearInterval(timer);
      for (const source of consumers.values()) {
        source.consumer.close();
      }
      consumers.clear();
      setStreams({});
      setError(null);
      transportRef.current?.close();
      transportRef.current = null;
      transportPromiseRef.current = null;
      deviceRef.current = null;
    };
  }, [active, actorId, consume, failureText, publishStreams, request, roomId]);

  return { streams, error, transportRef };
}
