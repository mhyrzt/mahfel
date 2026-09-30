"use client";

import { useEffect, useRef, useState } from "react";
import type { ReplayInput, VideoTrack } from "./replay/types";
import { createWorkspaceRenderer } from "./renderer/workspace";
import "./renderer/renderer.css";

export function RecordingScenePreview({
  input,
  timestamp = "0",
  tracks = [],
}: Readonly<{ input: ReplayInput; timestamp?: string; tracks?: VideoTrack[] }>) {
  const root = useRef<HTMLDivElement>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    if (!root.current) {
      return undefined;
    }
    setFailed(false);
    const renderer = createWorkspaceRenderer(root.current, "/recording-renderer");
    const media = document.createElement("div");
    root.current.prepend(media);
    let active = true;
    void renderer
      .initialize(input)
      .then(async () => {
        if (!active) {
          return;
        }
        const response = await renderer.render(timestamp, tracks, 960, 540);
        if (!active) {
          return;
        }
        media.replaceChildren(
          ...response.scene.layers
            .filter((layer) => layer.source !== "workspace")
            .map((layer) => {
              const tile = document.createElement("div");
              tile.className = "native-media-preview";
              tile.style.cssText = `position:absolute;left:${layer.x}px;top:${layer.y}px;width:${layer.width}px;height:${layer.height}px`;
              tile.textContent = "Camera preview";
              return tile;
            }),
        );
      })
      .catch(() => {
        if (active) {
          setFailed(true);
        }
      });
    return () => {
      active = false;
      renderer.destroy();
    };
  }, [input, timestamp, tracks]);
  return (
    <div
      className="relative w-[960px] max-w-full overflow-hidden rounded-xl bg-slate-950"
      style={{ aspectRatio: "16/9" }}
    >
      <div ref={root} className="recording-scene relative" />
      {failed ? <p role="alert">Recording preview unavailable</p> : null}
    </div>
  );
}
