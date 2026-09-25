import { expect, test } from "vitest";
import * as Y from "yjs";

import { readElements } from "../board-elements";

test("only renders an image after its stable asset reference is synced", () => {
  const doc = new Y.Doc();
  const elements = doc.getMap<string>("elements");
  const assets = doc.getMap<string>("image-assets");
  elements.set(
    "image-1",
    JSON.stringify({ id: "image-1", type: "image", fileId: "excalidraw-1", x: 0, y: 0 }),
  );
  expect(readElements(elements, assets)).toEqual([]);
  assets.set("excalidraw-1", "api-file-1");
  expect(readElements(elements, assets)).toHaveLength(1);
});

test("ignores unsupported or malformed whiteboard objects", () => {
  const doc = new Y.Doc();
  const elements = doc.getMap<string>("elements");
  const assets = doc.getMap<string>("image-assets");
  elements.set(
    "bad",
    JSON.stringify({ id: "bad", type: "image", fileId: "file", x: Infinity, y: 0 }),
  );
  elements.set("embed", JSON.stringify({ id: "embed", type: "embeddable", x: 0, y: 0 }));
  expect(readElements(elements, assets)).toEqual([]);
});
