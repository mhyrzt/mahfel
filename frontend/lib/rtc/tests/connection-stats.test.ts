import { expect, test } from "vitest";

import {
  connectionQuality,
  readConnectionStats,
  emptyConnectionMetrics,
  combineConnectionMetrics,
} from "../connection-stats";

function report(entries: Array<RTCStats & Record<string, unknown>>): RTCStatsReport {
  return new Map(entries.map((entry) => [entry.id, entry]));
}

test("reads RTT from the selected pair and ignores other successful candidates", () => {
  const { metrics } = readConnectionStats(
    report([
      { id: "transport", type: "transport", timestamp: 1000, selectedCandidatePairId: "selected" },
      {
        id: "selected",
        type: "candidate-pair",
        timestamp: 1000,
        state: "succeeded",
        currentRoundTripTime: 0.052,
      },
      {
        id: "other",
        type: "candidate-pair",
        timestamp: 1000,
        state: "succeeded",
        nominated: true,
        currentRoundTripTime: 9,
      },
    ]),
    new Map(),
  );
  expect(metrics.latencyMs).toBe(52);
  expect(metrics.packetLossPercent).toBeNull();
});

test("packet loss describes the recent interval rather than lifetime counters", () => {
  const first = readConnectionStats(
    report([
      {
        id: "audio",
        type: "inbound-rtp",
        timestamp: 1000,
        packetsReceived: 1000,
        packetsLost: 100,
        jitter: 0.015,
      },
    ]),
    new Map(),
  );
  expect(first.metrics.packetLossPercent).toBeNull();
  const next = readConnectionStats(
    report([
      {
        id: "audio",
        type: "inbound-rtp",
        timestamp: 4000,
        packetsReceived: 1098,
        packetsLost: 102,
        jitter: 0.015,
      },
    ]),
    first.packets,
  );
  expect(next.metrics.packetLossPercent).toBe(2);
  expect(next.metrics.jitterMs).toBe(15);
  expect(connectionQuality(next.metrics)).toBe("fair");
});

test("missing, cached, reset and idle measurements are not invented", () => {
  const before = new Map([["audio", { received: 100, lost: 10, timestamp: 1000 }]]);
  for (const entry of [
    { packetsReceived: 100, packetsLost: 10, timestamp: 2000 },
    { packetsReceived: 120, packetsLost: 12, timestamp: 1000 },
    { packetsReceived: 10, packetsLost: 1, timestamp: 2000 },
  ]) {
    const result = readConnectionStats(
      report([{ id: "audio", type: "inbound-rtp", jitter: 0.1, ...entry }]),
      before,
    );
    expect(result.metrics).toEqual(emptyConnectionMetrics);
  }
  expect(
    readConnectionStats(
      report([
        {
          id: "pair",
          type: "candidate-pair",
          timestamp: 1000,
          state: "succeeded",
          nominated: true,
          currentRoundTripTime: NaN,
        },
      ]),
      new Map(),
    ).metrics,
  ).toEqual(emptyConnectionMetrics);
  expect(connectionQuality(emptyConnectionMetrics)).toBe("unknown");
});

test("quality uses the worst available transport measurement", () => {
  const metrics = combineConnectionMetrics([
    { latencyMs: 50, jitterMs: 10, packetLossPercent: 0 },
    { latencyMs: 800, jitterMs: null, packetLossPercent: null },
  ]);
  expect(metrics).toEqual({ latencyMs: 800, jitterMs: 10, packetLossPercent: 0 });
  expect(connectionQuality(metrics)).toBe("poor");
  expect(connectionQuality({ latencyMs: 50, jitterMs: null, packetLossPercent: null })).toBe(
    "good",
  );
});

test("outgoing loss and jitter come from recent remote receiver reports", () => {
  const entries = [
    {
      id: "remote",
      type: "remote-inbound-rtp" as const,
      timestamp: Date.now(),
      fractionLost: 0.06,
      jitter: 0.09,
    },
  ];
  const result = readConnectionStats(report(entries), new Map());
  expect(result.metrics.packetLossPercent).toBe(6);
  expect(result.metrics.jitterMs).toBe(90);
  expect(connectionQuality(result.metrics)).toBe("poor");
  entries[0].timestamp -= 16_000;
  expect(readConnectionStats(report(entries), new Map()).metrics).toEqual(emptyConnectionMetrics);
});
