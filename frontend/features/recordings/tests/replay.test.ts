import { describe, expect, test } from "vitest";
import * as Y from "yjs";
import { ReplayState, postgresBytes } from "../replay/state";
import { describeScene } from "../replay/scene";
import { parseReplayInput } from "../replay/types";
import { replayFixture } from "./replay-fixtures";

describe("recording replay", () => {
  test("restores workspace, preserves pause duration and reloads resume snapshots", () => {
    const state = new ReplayState(replayFixture());
    state.advance(BigInt("2000000000"));
    expect(state.surface).toBe("whiteboard");
    expect(state.toasts).toHaveLength(2);
    expect(state.document("whiteboard")?.getMap("elements").size).toBe(1);
    state.advance(BigInt("11000000000"));
    expect(state.slate).toBe("paused");
    expect(state.toasts).toHaveLength(0);
    state.advance(BigInt("13000000000"));
    expect(state.slate).toBeNull();
    expect(state.generation).toBe(2);
    expect(state.document("whiteboard")?.getMap("elements").size).toBe(1);
    state.advance(BigInt("2000000000"));
    expect(state.generation).toBe(1);
    expect(state.toasts).toHaveLength(2);
    state.destroy();
  });
  test("chat edits change visible messages and deletion removes them", () => {
    const input = replayFixture();
    input.events.push(
      {
        generation: 1,
        event_kind: "chat",
        source_id: "edit",
        at_ns: "7000000000",
        payload: { kind: "edited", message_id: "first", body: "Updated message" },
      },
      {
        generation: 1,
        event_kind: "chat",
        source_id: "delete",
        at_ns: "8000000000",
        payload: { kind: "deleted", message_id: "second" },
      },
    );
    const state = new ReplayState(input);
    state.advance(BigInt("2500000000"));
    expect(state.toasts[0].body).toBe("Updated message");
    state.advance(BigInt("3500000000"));
    expect(state.toasts).toHaveLength(1);
    state.advance(BigInt("7000000000"));
    expect(state.toasts).toHaveLength(0);
    state.destroy();
  });
  test("chat bursts keep at most three toasts without affecting the main surface", () => {
    const input = replayFixture();
    input.events = Array.from({ length: 5 }, (_, index) => ({
      generation: 1,
      event_kind: "chat",
      source_id: `${index}`,
      at_ns: `${6000000000 + index}`,
      payload: {
        kind: "created",
        message_id: `${index}`,
        author_actor_id: "mahyar",
        body: `Message ${index}`,
      },
    }));
    const state = new ReplayState(input);
    state.advance(BigInt("2000000000"));
    expect(state.toasts.map((item) => item.id)).toEqual(["2", "3", "4"]);
    expect(state.surface).toBe("whiteboard");
    state.destroy();
  });
  test("rotates camera pages and gives screen sharing priority", () => {
    const state = new ReplayState(replayFixture());
    state.advance(BigInt("13000000000"));
    const tracks = Array.from({ length: 14 }, (_, index) => ({
      id: `camera-${index}`,
      actorId: `${index}`.padStart(2, "0"),
      source: "camera" as const,
    }));
    const first = describeScene(state, BigInt(0), tracks);
    const second = describeScene(state, BigInt("10000000000"), tracks);
    expect(first.cameraPages).toBe(3);
    expect(second.cameraPage).toBe(1);
    expect(first.layers.filter((item) => item.fit === "cover")).toHaveLength(6);
    const screen = describeScene(state, BigInt(0), [
      ...tracks,
      { id: "screen", actorId: "actor", source: "screen" },
    ]);
    expect(screen.surface).toBeNull();
    expect(screen.layers[0].source).toBe("screen");
    state.destroy();
  });
  test("document deletions invalidate the workspace frame cache", () => {
    const input = replayFixture();
    const doc = new Y.Doc();
    const checkpoint = input.snapshots.find((item) => item.snapshot_kind === "checkpoint");
    Y.applyUpdate(doc, postgresBytes(checkpoint?.payload.update_bytes));
    const before = Y.encodeStateVector(doc);
    doc.getMap("elements").delete("rectangle");
    expect(Y.encodeStateVector(doc)).toEqual(before);
    const bytes = `\\x${Array.from(Y.encodeStateAsUpdate(doc), (value) => value.toString(16).padStart(2, "0")).join("")}`;
    input.events.push({
      generation: 1,
      event_kind: "collaboration",
      source_id: "deletion",
      at_ns: "8000000000",
      payload: { kind: "document_update", document_id: "board", update_bytes: bytes },
    });
    const state = new ReplayState(input);
    state.advance(BigInt("1000000000"));
    const initial = state.documents.get("board")?.revision;
    state.advance(BigInt("4000000000"));
    expect(state.document("whiteboard")?.getMap("elements").size).toBe(0);
    expect(state.documents.get("board")?.revision).toBeGreaterThan(initial ?? 0);
    state.destroy();
    doc.destroy();
  });
  test("the newest screen share wins independently of actor sorting", () => {
    const state = new ReplayState(replayFixture());
    state.advance(BigInt(0));
    const scene = describeScene(state, BigInt(0), [
      { id: "old", actorId: "z", source: "screen", started_ns: "1" },
      { id: "new", actorId: "a", source: "screen", started_ns: "2" },
    ]);
    expect(scene.layers[0].source).toBe("new");
    state.destroy();
  });
  test("rejects invalid metadata and recorded binary encoding", () => {
    expect(() => parseReplayInput({ ...replayFixture(), version: 2 })).toThrow("Unsupported");
    expect(() => postgresBytes("oops")).toThrow("Invalid");
    expect(Array.from(postgresBytes("\\x00ff"))).toEqual([0, 255]);
  });
});
