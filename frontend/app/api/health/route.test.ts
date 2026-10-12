import { afterEach, describe, expect, it, vi } from "vitest";
import { name, version } from "@/package.json";
import { GET } from "./route";

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllEnvs();
  vi.useRealTimers();
});

describe("frontend health", () => {
  it("preserves the probe contract and reports the packaged app version", async () => {
    vi.stubEnv("NODE_ENV", "production");
    vi.spyOn(process, "uptime").mockReturnValue(123.9);
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-12T12:00:00.000Z"));
    const response = GET();
    expect(response.status).toBe(200);
    expect(response.headers.get("Cache-Control")).toBe("no-store");
    expect(await response.json()).toEqual({
      status: "ok",
      service: name,
      version,
      kind: "liveness",
      timestamp: "2026-10-12T12:00:00.000Z",
      uptimeSeconds: 123,
      environment: "production",
    });
  });

  it("refreshes its timestamp and process uptime for every request", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-12T12:00:00.000Z"));
    const uptime = vi.spyOn(process, "uptime").mockReturnValue(1);
    const first = await GET().json();
    vi.advanceTimersByTime(2000);
    uptime.mockReturnValue(3);
    const second = await GET().json();
    expect(second.timestamp).not.toBe(first.timestamp);
    expect(second.uptimeSeconds - first.uptimeSeconds).toBe(2);
  });

  it("uses unknown when the environment is unset", async () => {
    vi.stubEnv("NODE_ENV", undefined);
    expect((await GET().json()).environment).toBe("unknown");
  });
});
