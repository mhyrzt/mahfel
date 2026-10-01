import { expect, test, vi } from "vitest";
import { activeSegments, MediaPlayback, videoTracks, type MediaSegment } from "../media-playback";
import type { SceneV1 } from "../replay/types";

const segments: MediaSegment[] = [
  {
    track_id: "camera-1",
    actor_id: "actor-1",
    source: "camera",
    audio: false,
    start_ns: "0",
    end_ns: "1000000000",
    object_key: "first.mp4",
  },
  {
    track_id: "camera-1",
    actor_id: "actor-1",
    source: "camera",
    audio: false,
    start_ns: "2000000000",
    end_ns: "3000000000",
    object_key: "second.mp4",
  },
  {
    track_id: "mic-1",
    actor_id: "actor-1",
    source: "microphone",
    audio: true,
    start_ns: "0",
    end_ns: "3000000000",
    object_key: "audio.mp4",
  },
];

test("media follows the session clock and leaves capture gaps empty", () => {
  expect(videoTracks(segments, 500).map((track) => track.id)).toEqual(["camera-1"]);
  expect(videoTracks(segments, 1500)).toEqual([]);
  expect(activeSegments(segments, 1500).map((segment) => segment.track_id)).toEqual(["mic-1"]);
  expect(activeSegments(segments, 2500).map((segment) => segment.object_key)).toEqual([
    "second.mp4",
    "audio.mp4",
  ]);
});

test("clock updates keep an already playing track running until paused", async () => {
  const pause = vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
  const paused = vi.spyOn(HTMLMediaElement.prototype, "paused", "get").mockReturnValue(false);
  const root = document.createElement("div");
  const controller = new MediaPlayback(
    root,
    [segments[0]],
    async () => ({
      "first.mp4": "https://storage.test/first.mp4",
    }),
    vi.fn(),
  );
  const scene: SceneV1 = {
    version: 1,
    timestamp_ns: "0",
    canvas: { width: 960, height: 540 },
    surface: null,
    slate: null,
    layers: [],
    cameraPage: 0,
    cameraPages: 1,
  };
  controller.update(100, true, scene);
  await Promise.resolve();
  controller.update(200, true, scene);
  expect(root.querySelector("video")).not.toBeNull();
  expect(pause).not.toHaveBeenCalled();
  controller.update(300, false, scene);
  expect(pause).toHaveBeenCalledOnce();
  pause.mockRestore();
  paused.mockRestore();
});
