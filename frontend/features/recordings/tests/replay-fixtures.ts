import * as Y from "yjs";
import type { ReplayInput } from "../replay/types";

export function replayFixture(): ReplayInput {
  const doc = new Y.Doc();
  doc.getMap<string>("elements").set(
    "rectangle",
    JSON.stringify({
      id: "rectangle",
      type: "rectangle",
      x: 100,
      y: 100,
      width: 500,
      height: 250,
      angle: 0,
      strokeColor: "#6952e8",
      backgroundColor: "#efecff",
      fillStyle: "solid",
      strokeWidth: 2,
      strokeStyle: "solid",
      roughness: 1,
      opacity: 100,
      groupIds: [],
      seed: 1,
      version: 1,
      versionNonce: 1,
      isDeleted: false,
      boundElements: null,
      updated: 0,
      link: null,
      locked: false,
      index: "a0",
    }),
  );
  const bytes = `\\x${Array.from(Y.encodeStateAsUpdate(doc), (value) => value.toString(16).padStart(2, "0")).join("")}`;
  doc.destroy();
  return {
    version: 1,
    start_ns: "5000000000",
    duration_ns: "20000000000",
    intervals: [
      { generation: 1, start_ns: "5000000000", end_ns: "15000000000" },
      { generation: 2, start_ns: "17000000000", end_ns: "25000000000" },
    ],
    assets: [],
    actors: { mahyar: "Mahyar", sara: "سارا" },
    snapshots: [1, 2].flatMap((generation) => [
      {
        generation,
        snapshot_kind: "document",
        source_id: "board",
        payload: { kind: "whiteboard" },
      },
      {
        generation,
        snapshot_kind: "checkpoint",
        source_id: "board",
        payload: { document_id: "board", update_bytes: bytes },
      },
    ]),
    events: [
      {
        generation: 1,
        event_kind: "chat",
        source_id: "a",
        at_ns: "6000000000",
        payload: {
          kind: "created",
          message_id: "first",
          author_actor_id: "mahyar",
          body: "The updated diagram looks good.",
        },
      },
      {
        generation: 1,
        event_kind: "chat",
        source_id: "b",
        at_ns: "6500000000",
        payload: {
          kind: "created",
          message_id: "second",
          author_actor_id: "sara",
          body: "عالیه، ممنون!",
        },
      },
    ],
  };
}
