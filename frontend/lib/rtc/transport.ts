"use client";

import type { Device } from "mediasoup-client";

export type RtcTransport = ReturnType<Device["createSendTransport"]>;

export interface TransportParams {
  id: string;
  iceParameters: unknown;
  iceCandidates: unknown;
  dtlsParameters: unknown;
}

export function createSendTransport(
  device: Device,
  params: TransportParams,
  onConnect: (dtlsParameters: unknown) => Promise<void>,
  onProduce: (kind: string, rtpParameters: unknown) => Promise<string>,
): RtcTransport {
  const transport = device.createSendTransport({
    id: params.id,
    iceParameters: params.iceParameters as never,
    iceCandidates: params.iceCandidates as never,
    dtlsParameters: params.dtlsParameters as never,
  });

  transport.on("connect", ({ dtlsParameters }, callback, errback) => {
    onConnect(dtlsParameters)
      .then(() => callback())
      .catch(errback);
  });

  transport.on("produce", ({ kind, rtpParameters }, callback, errback) => {
    onProduce(kind, rtpParameters)
      .then((id) => callback({ id }))
      .catch(errback);
  });

  transport.on("connectionstatechange", (state) => {
    if (state === "failed") {
      transport.restartIce({ iceParameters: params.iceParameters as never });
    }
  });

  return transport;
}

export function createRecvTransport(
  device: Device,
  params: TransportParams,
  onConnect: (dtlsParameters: unknown) => Promise<void>,
): RtcTransport {
  const transport = device.createRecvTransport({
    id: params.id,
    iceParameters: params.iceParameters as never,
    iceCandidates: params.iceCandidates as never,
    dtlsParameters: params.dtlsParameters as never,
  });

  transport.on("connect", ({ dtlsParameters }, callback, errback) => {
    onConnect(dtlsParameters)
      .then(() => callback())
      .catch(errback);
  });

  transport.on("connectionstatechange", (state) => {
    if (state === "failed") {
      transport.restartIce({ iceParameters: params.iceParameters as never });
    }
  });

  return transport;
}
