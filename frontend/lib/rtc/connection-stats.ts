export type ConnectionQuality = "good" | "fair" | "poor" | "unknown";

export interface ConnectionMetrics {
  latencyMs: number | null;
  jitterMs: number | null;
  packetLossPercent: number | null;
}

export interface PacketSample {
  received: number;
  lost: number;
  timestamp: number;
}

export const emptyConnectionMetrics: ConnectionMetrics = {
  latencyMs: null,
  jitterMs: null,
  packetLossPercent: null,
};

function record(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null ? (value as Record<string, unknown>) : null;
}

function number(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function maximum(values: number[]): number | null {
  return values.length ? Math.max(...values) : null;
}

export function readConnectionStats(
  report: RTCStatsReport,
  previous: ReadonlyMap<string, PacketSample>,
): { metrics: ConnectionMetrics; packets: Map<string, PacketSample> } {
  const entries: Record<string, unknown>[] = [];
  report.forEach((value: unknown) => {
    const entry = record(value);
    if (entry) {
      entries.push(entry);
    }
  });
  const selectedIds = new Set(
    entries
      .filter((entry) => entry.type === "transport")
      .map((entry) => entry.selectedCandidatePairId)
      .filter((id): id is string => typeof id === "string"),
  );
  const latencies: number[] = [];
  const jitters: number[] = [];
  const losses: number[] = [];
  const packets = new Map<string, PacketSample>();
  for (const entry of entries) {
    if (
      entry.type === "candidate-pair" &&
      entry.state === "succeeded" &&
      (selectedIds.size ? selectedIds.has(String(entry.id)) : entry.nominated === true)
    ) {
      const rtt = number(entry.currentRoundTripTime);
      if (rtt !== null && rtt >= 0) {
        latencies.push(rtt * 1_000);
      }
    }
    if (entry.type !== "inbound-rtp" && entry.type !== "remote-inbound-rtp") {
      continue;
    }
    const jitter = number(entry.jitter);
    const received = number(entry.packetsReceived);
    const lost = number(entry.packetsLost);
    const timestamp = number(entry.timestamp);
    if (entry.type === "remote-inbound-rtp") {
      const fractionLost = number(entry.fractionLost);
      if (timestamp !== null && Date.now() - timestamp <= 15_000 && Date.now() >= timestamp) {
        if (fractionLost !== null && fractionLost >= 0 && fractionLost <= 1) {
          losses.push(fractionLost * 100);
        }
        if (jitter !== null && jitter >= 0) {
          jitters.push(jitter * 1_000);
        }
      }
      continue;
    }
    if (typeof entry.id !== "string" || received === null || lost === null || timestamp === null) {
      continue;
    }
    packets.set(entry.id, { received, lost, timestamp });
    const before = previous.get(entry.id);
    if (!before || timestamp <= before.timestamp || received < before.received) {
      continue;
    }
    const receivedDelta = received - before.received;
    const lostDelta = Math.max(0, lost - before.lost);
    if (receivedDelta + lostDelta > 0) {
      losses.push((lostDelta / (receivedDelta + lostDelta)) * 100);
      if (jitter !== null && jitter >= 0) {
        jitters.push(jitter * 1_000);
      }
    }
  }
  return {
    metrics: {
      latencyMs: maximum(latencies),
      jitterMs: maximum(jitters),
      packetLossPercent: maximum(losses),
    },
    packets,
  };
}

export function connectionQuality(metrics: ConnectionMetrics): ConnectionQuality {
  const { latencyMs, jitterMs, packetLossPercent } = metrics;
  if ((latencyMs ?? 0) >= 700 || (jitterMs ?? 0) >= 80 || (packetLossPercent ?? 0) >= 5) {
    return "poor";
  }
  if ((latencyMs ?? 0) >= 300 || (jitterMs ?? 0) >= 30 || (packetLossPercent ?? 0) >= 2) {
    return "fair";
  }
  return Object.values(metrics).some((value) => value !== null) ? "good" : "unknown";
}

export function combineConnectionMetrics(metrics: ConnectionMetrics[]): ConnectionMetrics {
  const worst = (key: keyof ConnectionMetrics) =>
    maximum(metrics.flatMap((item) => (item[key] === null ? [] : [item[key]])));
  return {
    latencyMs: worst("latencyMs"),
    jitterMs: worst("jitterMs"),
    packetLossPercent: worst("packetLossPercent"),
  };
}
