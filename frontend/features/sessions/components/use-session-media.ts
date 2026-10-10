"use client";

import { useCallback, useEffect, useRef, useState, type RefObject } from "react";
import type { Producer } from "mediasoup-client/types";

import { roomCopy } from "@/features/rooms/room-copy";
import { createDevice, isDeviceSupported } from "@/lib/rtc/device";
import { createSendTransport, type RtcTransport } from "@/lib/rtc/transport";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import type { SignalingClient } from "@/lib/websocket/client";

type Source = "microphone" | "camera" | "screen";
type ActiveSource = { producer: Producer; stream: MediaStream };

export function useSessionMedia(
  clientRef: RefObject<SignalingClient | null>,
  roomId: string,
  locale: Locale,
  active = true,
  onFailure?: () => void,
) {
  const [enabled, setEnabled] = useState<Record<Source, boolean>>({
    microphone: false,
    camera: false,
    screen: false,
  });
  const [streams, setStreams] = useState<{ camera?: MediaStream; screen?: MediaStream }>({});
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const transportRef = useRef<RtcTransport | null>(null);
  const transportPromiseRef = useRef<Promise<RtcTransport> | null>(null);
  const generationRef = useRef(0);
  const mountedRef = useRef(true);
  const busyRef = useRef(false);
  const sourcesRef = useRef<Partial<Record<Source, ActiveSource>>>({});
  const failureText = roomCopy(locale).mediaUnavailable;
  const permissionText = messagesFor(locale).account.previewDenied;

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

  const ensureTransport = useCallback(async () => {
    if (transportRef.current && !transportRef.current.closed) {
      return transportRef.current;
    }
    if (transportPromiseRef.current) {
      return transportPromiseRef.current;
    }
    const generation = generationRef.current;
    const client = clientRef.current;
    const isCurrent = () =>
      mountedRef.current && generation === generationRef.current && client === clientRef.current;
    const pending = (async () => {
      if (!isDeviceSupported()) {
        throw new Error("WebRTC is unsupported");
      }
      const response = await request({ type: "create_transport", room_id: roomId });
      if (response.type !== "transport_created" || !response.rtp_capabilities) {
        throw new Error("Media transport parameters are unavailable");
      }
      if (!isCurrent()) {
        throw new Error("Media setup cancelled");
      }
      const device = await createDevice(JSON.parse(response.rtp_capabilities));
      if (!isCurrent()) {
        throw new Error("Media setup cancelled");
      }
      const transport = createSendTransport(
        device,
        {
          id: response.transport_id,
          iceParameters: JSON.parse(response.ice_parameters),
          iceCandidates: JSON.parse(response.ice_candidates),
          dtlsParameters: JSON.parse(response.dtls_parameters),
        },
        async (dtlsParameters) => {
          if (!isCurrent()) {
            throw new Error("Media setup cancelled");
          }
          const connected = await request({
            type: "connect_transport",
            room_id: roomId,
            transport_id: response.transport_id,
            dtls_parameters: JSON.stringify(dtlsParameters),
          });
          if (connected.type !== "transport_connected") {
            throw new Error("Media transport connection failed");
          }
        },
        async (kind, rtpParameters, source) => {
          if (!isCurrent()) {
            throw new Error("Media setup cancelled");
          }
          const produced = await request({
            type: "produce",
            room_id: roomId,
            transport_id: response.transport_id,
            kind,
            rtp_parameters: JSON.stringify(rtpParameters),
            source,
          });
          if (produced.type !== "produced") {
            throw new Error("Media publication failed");
          }
          return produced.producer_id;
        },
      );
      transport.on("connectionstatechange", (state) => {
        if (!isCurrent()) {
          return;
        }
        if (state === "failed" || state === "disconnected") {
          setError(failureText);
          if (state === "failed") {
            onFailure?.();
          }
        } else if (state === "connected") {
          setError(null);
        }
      });
      transportRef.current = transport;
      return transport;
    })();
    transportPromiseRef.current = pending;
    try {
      return await pending;
    } finally {
      if (transportPromiseRef.current === pending) {
        transportPromiseRef.current = null;
      }
    }
  }, [clientRef, failureText, onFailure, request, roomId]);

  const stop = useCallback(
    async (source: Source) => {
      const active = sourcesRef.current[source];
      if (!active) {
        return;
      }
      delete sourcesRef.current[source];
      active.producer.close();
      active.stream.getTracks().forEach((track) => track.stop());
      setEnabled((current) => ({ ...current, [source]: false }));
      setStreams((current) => ({ ...current, [source]: undefined }));
      try {
        await request({ type: "stop_source", room_id: roomId, source });
      } catch {
        setError(failureText);
      }
    },
    [failureText, request, roomId],
  );

  const toggle = useCallback(
    async (source: Source) => {
      if (busyRef.current || !mountedRef.current) {
        return;
      }
      busyRef.current = true;
      const generation = generationRef.current;
      const client = clientRef.current;
      const isCurrent = () =>
        mountedRef.current && generation === generationRef.current && client === clientRef.current;
      setBusy(true);
      setError(null);
      try {
        if (sourcesRef.current[source]) {
          await stop(source);
          return;
        }
        if (source !== "screen" && navigator.permissions?.query) {
          const permission = await navigator.permissions
            .query({
              name: (source === "microphone" ? "microphone" : "camera") as PermissionName,
            })
            .catch(() => null);
          if (permission?.state === "denied") {
            throw new DOMException("Device permission denied", "NotAllowedError");
          }
          if (!isCurrent()) {
            return;
          }
        }
        const stream =
          source === "screen"
            ? await navigator.mediaDevices.getDisplayMedia({ video: true, audio: false })
            : await navigator.mediaDevices.getUserMedia({
                audio: source === "microphone",
                video: source === "camera",
              });
        if (!isCurrent()) {
          stream.getTracks().forEach((item) => item.stop());
          return;
        }
        const track =
          source === "microphone" ? stream.getAudioTracks()[0] : stream.getVideoTracks()[0];
        if (!track) {
          stream.getTracks().forEach((item) => item.stop());
          throw new Error("No media track was available");
        }
        try {
          const transport = await ensureTransport();
          if (!isCurrent()) {
            throw new Error("Media setup cancelled");
          }
          const producer = await transport.produce({ track, appData: { source } });
          if (!isCurrent()) {
            producer.close();
            stream.getTracks().forEach((item) => item.stop());
            return;
          }
          sourcesRef.current[source] = { producer, stream };
          setEnabled((current) => ({ ...current, [source]: true }));
          if (source !== "microphone") {
            setStreams((current) => ({ ...current, [source]: stream }));
          }
          if (source === "screen") {
            track.addEventListener("ended", () => void stop("screen"), { once: true });
          }
        } catch (error) {
          stream.getTracks().forEach((item) => item.stop());
          throw error;
        }
      } catch (error) {
        if (!isCurrent()) {
          return;
        }
        setError(
          error instanceof DOMException && error.name === "NotAllowedError"
            ? permissionText
            : failureText,
        );
      } finally {
        if (mountedRef.current && generation === generationRef.current) {
          busyRef.current = false;
          setBusy(false);
        }
      }
    },
    [clientRef, ensureTransport, failureText, permissionText, stop],
  );

  useEffect(() => {
    mountedRef.current = active;
    const sources = sourcesRef.current;
    return () => {
      mountedRef.current = false;
      generationRef.current += 1;
      busyRef.current = false;
      Object.values(sources).forEach((active) => {
        active.producer.close();
        active.stream.getTracks().forEach((track) => track.stop());
      });
      transportRef.current?.close();
      transportRef.current = null;
      transportPromiseRef.current = null;
      sourcesRef.current = {};
      setEnabled({ microphone: false, camera: false, screen: false });
      setStreams({});
      setBusy(false);
    };
  }, [roomId, active]);

  return { enabled, streams, busy, error, toggle, stop, transportRef };
}
