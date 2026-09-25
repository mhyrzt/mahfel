import { expect, test } from "vitest";
import { createEditor } from "lexical";

import { NoteImageNode } from "../components/note-image-node";

test("a notes image serializes only its stable file reference", () => {
  const editor = createEditor({ nodes: [NoteImageNode] });
  let serialized = "";
  editor.update(() => {
    const image = NoteImageNode.importJSON({
      type: "collaboration-image",
      version: 1,
      fileId: "api-file-1",
      altText: "Workshop diagram",
    });
    serialized = JSON.stringify(image.exportJSON());
  });
  expect(JSON.parse(serialized)).toMatchObject({
    type: "collaboration-image",
    fileId: "api-file-1",
    altText: "Workshop diagram",
  });
  expect(serialized).not.toContain("data:image/");
});
