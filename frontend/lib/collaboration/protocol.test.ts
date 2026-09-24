import { afterEach, describe, expect, it } from "vitest";
import * as awarenessProtocol from "y-protocols/awareness";
import * as Y from "yjs";

import { awarenessFrame, receiveFrame, updateFrame } from "./protocol";
import type { MahfelCollaborationProvider } from "./provider";

const origin = {} as MahfelCollaborationProvider;
const resources: { docs: Y.Doc[]; awareness: awarenessProtocol.Awareness[] } = {
  docs: [],
  awareness: [],
};

afterEach(() => {
  resources.awareness.splice(0).forEach((item) => item.destroy());
  resources.docs.splice(0).forEach((item) => item.destroy());
});

function doc() {
  const value = new Y.Doc();
  resources.docs.push(value);
  return value;
}

function awareness(value: Y.Doc) {
  const state = new awarenessProtocol.Awareness(value);
  resources.awareness.push(state);
  return state;
}

describe("collaboration protocol", () => {
  it("applies a Yjs binary document update", () => {
    const source = doc();
    const target = doc();
    source.getText("notes").insert(0, "Together");
    receiveFrame(updateFrame(Y.encodeStateAsUpdate(source)), target, awareness(target), origin);
    expect(target.getText("notes").toString()).toBe("Together");
  });

  it("relays awareness and removes it after a disconnect frame", () => {
    const source = doc();
    const target = doc();
    const sender = awareness(source);
    const receiver = awareness(target);
    sender.setLocalStateField("cursor", { x: 10, y: 20 });
    receiveFrame(awarenessFrame(sender, [source.clientID]), target, receiver, origin);
    expect(receiver.getStates().get(source.clientID)?.cursor).toEqual({ x: 10, y: 20 });
    sender.setLocalState(null);
    receiveFrame(awarenessFrame(sender, [source.clientID]), target, receiver, origin);
    expect(receiver.getStates().has(source.clientID)).toBe(false);
  });
});
