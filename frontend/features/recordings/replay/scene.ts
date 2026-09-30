import type { ReplayState } from "./state";
import type { SceneV1, VideoTrack } from "./types";

export function describeScene(
  state: ReplayState,
  outputNs: bigint,
  tracks: VideoTrack[],
  width = 1920,
  height = 1080,
): SceneV1 {
  const sorted = [...tracks].sort(
    (a, b) => a.actorId.localeCompare(b.actorId) || a.id.localeCompare(b.id),
  );
  const screen = sorted
    .filter((item) => item.source === "screen")
    .sort((a, b) => {
      const difference = BigInt(a.started_ns ?? "0") - BigInt(b.started_ns ?? "0");
      if (difference !== BigInt(0)) {
        return difference < BigInt(0) ? -1 : 1;
      }
      return a.id.localeCompare(b.id);
    })
    .at(-1);
  const content = Boolean(screen || state.surface);
  const cameras = sorted.filter((item) => item.source === "camera");
  const perPage = content ? 6 : 9;
  const cameraPages = Math.max(1, Math.ceil(cameras.length / perPage));
  const cameraPage = Number((outputNs / BigInt("10000000000")) % BigInt(cameraPages));
  const selected = cameras.slice(cameraPage * perPage, (cameraPage + 1) * perPage);
  const layers: SceneV1["layers"] = [];
  if (!state.slate) {
    if (screen) {
      layers.push({
        source: screen.id,
        x: 0,
        y: 0,
        width,
        height: (height * 5) / 6,
        fit: "contain",
        z: 1,
      });
    }
    if (content && !screen) {
      layers.push({
        source: "workspace",
        x: 0,
        y: 0,
        width,
        height: (height * 5) / 6,
        fit: "contain",
        z: 2,
      });
    }
    selected.forEach((track, index) => {
      const columns = content ? 6 : 3;
      const tileWidth = width / columns;
      const tileHeight = content ? height / 6 : height / 3;
      layers.push({
        source: track.id,
        x: (index % columns) * tileWidth,
        y: content ? (height * 5) / 6 : Math.floor(index / columns) * tileHeight,
        width: tileWidth,
        height: tileHeight,
        fit: "cover",
        z: 1,
      });
    });
  }
  return {
    version: 1,
    timestamp_ns: outputNs.toString(),
    canvas: { width, height },
    surface: screen ? null : state.surface,
    slate: state.slate,
    layers,
    cameraPage,
    cameraPages,
  };
}
