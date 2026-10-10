import { act, renderHook } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import type { RefObject } from "react";
import type { SignalingClient } from "@/lib/websocket/client";

const { createDevice, createSendTransport } = vi.hoisted(() => ({
  createDevice: vi.fn(),
  createSendTransport: vi.fn(),
}));
vi.mock("@/lib/rtc/device", () => ({ createDevice, isDeviceSupported: () => true }));
vi.mock("@/lib/rtc/transport", () => ({ createSendTransport }));
import { useSessionMedia } from "../components/use-session-media";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

function streamFixture() {
  const track = { stop: vi.fn(), addEventListener: vi.fn() };
  const stream = {
    getTracks: () => [track],
    getAudioTracks: () => [track],
    getVideoTracks: () => [track],
  } as unknown as MediaStream;
  return { stream, track };
}

function mount(request = vi.fn()) {
  const clientRef = { current: { request } } as unknown as RefObject<SignalingClient | null>;
  return renderHook(() => useSessionMedia(clientRef, "room-1", "en"));
}

test("a capture permission result arriving after unmount is stopped without publishing", async () => {
  const { stream, track } = streamFixture();
  let resolveCapture!: (stream: MediaStream) => void;
  const getUserMedia = vi.fn(
    () =>
      new Promise<MediaStream>((resolve) => {
        resolveCapture = resolve;
      }),
  );
  vi.stubGlobal("navigator", { mediaDevices: { getUserMedia } });
  const request = vi.fn();
  const { result, unmount } = mount(request);
  let pending!: Promise<void>;
  act(() => {
    pending = result.current.toggle("microphone");
  });
  unmount();
  resolveCapture(stream);
  await pending;
  expect(track.stop).toHaveBeenCalled();
  expect(request).not.toHaveBeenCalled();
  expect(createSendTransport).not.toHaveBeenCalled();
});

test("a device load arriving after unmount cannot create a local transport", async () => {
  const { stream, track } = streamFixture();
  vi.stubGlobal("navigator", { mediaDevices: { getUserMedia: vi.fn().mockResolvedValue(stream) } });
  let resolveDevice!: (device: object) => void;
  createDevice.mockImplementation(
    () =>
      new Promise((resolve) => {
        resolveDevice = resolve;
      }),
  );
  const request = vi.fn().mockResolvedValue({
    type: "transport_created",
    transport_id: "send-1",
    rtp_capabilities: "{}",
    ice_parameters: "{}",
    ice_candidates: "[]",
    dtls_parameters: "{}",
  });
  const { result, unmount } = mount(request);
  let pending!: Promise<void>;
  await act(async () => {
    pending = result.current.toggle("camera");
    await Promise.resolve();
    await Promise.resolve();
  });
  expect(createDevice).toHaveBeenCalled();
  unmount();
  resolveDevice({});
  await pending;
  expect(track.stop).toHaveBeenCalled();
  expect(createSendTransport).not.toHaveBeenCalled();
});

test("a producer arriving after unmount is closed and its capture is stopped", async () => {
  const { stream, track } = streamFixture();
  vi.stubGlobal("navigator", { mediaDevices: { getUserMedia: vi.fn().mockResolvedValue(stream) } });
  createDevice.mockResolvedValue({});
  let resolveProducer!: (producer: object) => void;
  const produce = vi.fn(
    () =>
      new Promise((resolve) => {
        resolveProducer = resolve;
      }),
  );
  const closeTransport = vi.fn();
  createSendTransport.mockReturnValue({
    closed: false,
    on: vi.fn(),
    close: closeTransport,
    produce,
  });
  const request = vi.fn().mockResolvedValue({
    type: "transport_created",
    transport_id: "send-1",
    rtp_capabilities: "{}",
    ice_parameters: "{}",
    ice_candidates: "[]",
    dtls_parameters: "{}",
  });
  const { result, unmount } = mount(request);
  let pending!: Promise<void>;
  await act(async () => {
    pending = result.current.toggle("camera");
    await Promise.resolve();
    await Promise.resolve();
    await Promise.resolve();
  });
  expect(produce).toHaveBeenCalled();
  unmount();
  const closeProducer = vi.fn();
  resolveProducer({ close: closeProducer });
  await pending;
  expect(closeProducer).toHaveBeenCalled();
  expect(closeTransport).toHaveBeenCalled();
  expect(track.stop).toHaveBeenCalled();
});

test("a denied permission does not capture or allocate media and gives recovery guidance", async () => {
  const getUserMedia = vi.fn();
  vi.stubGlobal("navigator", {
    permissions: { query: vi.fn().mockResolvedValue({ state: "denied" }) },
    mediaDevices: { getUserMedia },
  });
  const request = vi.fn();
  const { result } = mount(request);
  await act(async () => {
    await result.current.toggle("microphone");
  });
  expect(getUserMedia).not.toHaveBeenCalled();
  expect(request).not.toHaveBeenCalled();
  expect(result.current.busy).toBe(false);
  expect(result.current.error).toBe(
    "Device access is unavailable. Check browser permissions and try again.",
  );
});
