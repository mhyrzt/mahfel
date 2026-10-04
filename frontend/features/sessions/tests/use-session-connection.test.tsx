import { act, renderHook } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

import type { RtcTransport } from "@/lib/rtc/transport";
import { emptyConnectionMetrics } from "@/lib/rtc/connection-stats";
import { useSessionConnection } from "../components/use-session-connection";

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

test("samples connected transports, clears disconnected values and cancels polling", async () => {
  vi.useFakeTimers();
  const getStats = vi.fn().mockResolvedValue(
    new Map([
      [
        "pair",
        {
          id: "pair",
          type: "candidate-pair",
          state: "succeeded",
          nominated: true,
          currentRoundTripTime: 0.04,
        },
      ],
    ]),
  );
  const transport = { connectionState: "connected", closed: false, getStats };
  const sendRef = { current: transport as unknown as RtcTransport };
  const receiveRef = { current: null };
  const { result, unmount } = renderHook(() =>
    useSessionConnection("connected", true, sendRef, receiveRef),
  );
  await act(async () => {});
  expect(result.current.metrics.latencyMs).toBe(40);
  transport.connectionState = "disconnected";
  await act(async () => {
    await vi.advanceTimersByTimeAsync(3000);
  });
  expect(result.current.media).toBe("disconnected");
  expect(result.current.metrics).toEqual(emptyConnectionMetrics);
  unmount();
  expect(vi.getTimerCount()).toBe(0);
});

test("offline state clears measurements and statistics failures stay unavailable", async () => {
  vi.useFakeTimers();
  const online = vi.spyOn(navigator, "onLine", "get").mockReturnValue(true);
  const getStats = vi.fn().mockRejectedValue(new Error("Not supported"));
  const sendRef = {
    current: { connectionState: "connected", closed: false, getStats } as unknown as RtcTransport,
  };
  const receiveRef = { current: null };
  const { result } = renderHook(() => useSessionConnection("connected", true, sendRef, receiveRef));
  await act(async () => {});
  expect(result.current.media).toBe("unavailable");
  expect(result.current.metrics).toEqual(emptyConnectionMetrics);
  online.mockReturnValue(false);
  act(() => {
    window.dispatchEvent(new Event("offline"));
  });
  expect(result.current.offline).toBe(true);
  expect(result.current.metrics).toEqual(emptyConnectionMetrics);
  expect(vi.getTimerCount()).toBe(0);
});

test("an in-flight sample cannot reschedule polling after unmount", async () => {
  vi.useFakeTimers();
  let resolve: (report: RTCStatsReport) => void = () => undefined;
  const pending = new Promise<RTCStatsReport>((done) => {
    resolve = done;
  });
  const getStats = vi.fn().mockReturnValue(pending);
  const sendRef = {
    current: { connectionState: "connected", closed: false, getStats } as unknown as RtcTransport,
  };
  const { unmount } = renderHook(() =>
    useSessionConnection("connected", true, sendRef, { current: null }),
  );
  unmount();
  await act(async () => {
    resolve(new Map());
  });
  expect(vi.getTimerCount()).toBe(0);
});
