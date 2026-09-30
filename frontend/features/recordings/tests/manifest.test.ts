import { afterEach, expect, test, vi } from "vitest";
import { loadReplayInput } from "../replay/manifest";

const manifestKey = "recordings/r/replay/1/manifest-h.json";

const pages: Record<string, unknown> = {
  "manifest-url": {
    version: 1,
    partial: false,
    recording: {
      started_at: "2026-09-29T08:00:00Z",
      stopped_at: "2026-09-29T08:00:10Z",
      session_origin_ns: "1000000000000",
    },
    sections: {
      snapshots: [{ object_key: "snapshots", sha256: "x", entries: 1 }],
      events: [{ object_key: "events", sha256: "x", entries: 1 }],
      assets: [{ object_key: "assets", sha256: "x", entries: 1 }],
      actors: [{ object_key: "actors", sha256: "x", entries: 1 }],
    },
    intervals: [{ generation: 1, start_ns: "0", end_ns: "10000000000", loss_reason: null }],
  },
  "snapshots-url": [{ generation: 1, snapshot_kind: "document", source_id: "doc", payload: {} }],
  "events-url": [
    { generation: 1, event_kind: "chat", source_id: "m1", at_ns: "1000", payload: {} },
  ],
  "assets-url": [{ file_id: "file", mime: "image/png", archive_key: "recordings/r/assets/file" }],
  "actors-url": [{ actor_id: "actor", display_name: "Ada" }],
};

afterEach(() => {
  vi.unstubAllGlobals();
});

test("replay descriptor resolves archived pages and asset urls", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string) => new Response(JSON.stringify(pages[url]), { status: 200 })),
  );
  const input = await loadReplayInput({
    manifest_key: manifestKey,
    urls: {
      [manifestKey]: "manifest-url",
      snapshots: "snapshots-url",
      events: "events-url",
      assets: "assets-url",
      actors: "actors-url",
    },
    assets: { file: "https://storage.invalid/asset" },
  });
  expect(input.duration_ns).toBe("10000000000");
  expect(input.snapshots).toHaveLength(1);
  expect(input.events[0]?.at_ns).toBe("1000");
  expect(input.assets[0]?.archive_key).toBe("https://storage.invalid/asset");
  expect(input.actors.actor).toBe("Ada");
});

test("a missing manifest url fails explicitly", async () => {
  await expect(
    loadReplayInput({ manifest_key: manifestKey, urls: {}, assets: {} }),
  ).rejects.toThrow("Replay manifest unavailable");
});
