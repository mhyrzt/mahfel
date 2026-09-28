import { renderHook, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import type { RefObject } from "react";

import type { SignalingClient } from "@/lib/websocket/client";

const { createDevice, createRecvTransport, closeConsumer, closeTransport } = vi.hoisted(() => ({
  createDevice: vi.fn(),
  createRecvTransport: vi.fn(),
  closeConsumer: vi.fn(),
  closeTransport: vi.fn(),
}));

vi.mock("@/lib/rtc/device", () => ({ createDevice }));
vi.mock("@/lib/rtc/transport", () => ({ createRecvTransport }));

import { useRemoteSessionMedia } from "../components/use-remote-session-media";

test("discovers, consumes, and resumes a remote camera producer", async () => {
  const track = { kind: "video" };
  vi.stubGlobal(
    "MediaStream",
    class {
      constructor(readonly tracks: unknown[]) {}
    },
  );
  createDevice.mockResolvedValue({ rtpCapabilities: { codecs: [] } });
  createRecvTransport.mockReturnValue({
    id: "receive-transport",
    closed: false,
    on: vi.fn(),
    close: closeTransport,
    consume: vi.fn().mockResolvedValue({ track, close: closeConsumer }),
  });
  const request = vi.fn(async (message: { type: string }) => {
    switch (message.type) {
      case "list_producers":
        return {
          type: "producer_snapshot",
          producers: [
            { producer_id: "camera-1", actor_id: "shinji", kind: "video", source: "camera" },
          ],
        };
      case "create_transport":
        return {
          type: "transport_created",
          transport_id: "receive-transport",
          rtp_capabilities: "{}",
          ice_parameters: "{}",
          ice_candidates: "[]",
          dtls_parameters: "{}",
        };
      case "consume":
        return {
          type: "consumed",
          consumer_id: "consumer-1",
          rtp_parameters: "{}",
          paused: true,
        };
      case "resume_consumer":
        return { type: "consumer_resumed" };
      default:
        throw new Error(`Unexpected request: ${message.type}`);
    }
  });
  const clientRef = { current: { request } } as unknown as RefObject<SignalingClient | null>;

  const { result, unmount } = renderHook(() =>
    useRemoteSessionMedia(clientRef, "room-1", "misato", true, "en"),
  );

  await waitFor(() => expect(result.current.streams.shinji?.camera).toBeDefined());
  expect(request).toHaveBeenCalledWith(
    expect.objectContaining({ type: "consume", producer_id: "camera-1" }),
  );
  expect(request).toHaveBeenCalledWith(
    expect.objectContaining({ type: "resume_consumer", consumer_id: "consumer-1" }),
  );

  unmount();
  expect(closeConsumer).toHaveBeenCalled();
  expect(closeTransport).toHaveBeenCalled();
  vi.unstubAllGlobals();
});
