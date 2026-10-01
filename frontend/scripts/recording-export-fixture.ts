import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { createHash } from "node:crypto";
import sharp from "sharp";
import { createEditor, $getRoot, $createParagraphNode, $createTextNode } from "lexical";
import { createYjsBinding, syncLexicalUpdateToYjs, type Provider } from "@lexical/yjs";
import * as Y from "yjs";
import { $createHeadingNode, $createQuoteNode, HeadingNode, QuoteNode } from "@lexical/rich-text";
import { $createListNode, $createListItemNode, ListNode, ListItemNode } from "@lexical/list";
import { replayFixture } from "../features/recordings/tests/replay-fixtures";

const directory = process.argv[2];
if (!directory) {
  throw new Error("Fixture directory required");
}
await mkdir(directory, { recursive: true });
const doc = new Y.Doc();
const editor = createEditor({
  namespace: "fixture",
  nodes: [HeadingNode, QuoteNode, ListNode, ListItemNode],
  onError: (error) => {
    throw error;
  },
});
const binding = createYjsBinding({
  editor,
  id: "fixture",
  doc,
  docMap: new Map([["fixture", doc]]),
});
const provider: Provider = {
  awareness: {
    getLocalState: () => null,
    getStates: () => new Map(),
    on() {},
    off() {},
    setLocalState() {},
    setLocalStateField() {},
  },
  connect() {},
  disconnect() {},
  on() {},
  off() {},
};
const stop = editor.registerUpdateListener(
  ({ prevEditorState, editorState, dirtyElements, dirtyLeaves, normalizedNodes, tags }) => {
    syncLexicalUpdateToYjs(
      binding,
      provider,
      prevEditorState,
      editorState,
      dirtyElements,
      dirtyLeaves,
      normalizedNodes,
      tags,
    );
  },
);
editor.update(
  () => {
    $getRoot().append(
      $createHeadingNode("h2").append($createTextNode("Archived notes")),
      $createParagraphNode()
        .setDirection("rtl")
        .append($createTextNode("یادداشت‌های جلسه").toggleFormat("bold")),
      $createQuoteNode().append($createTextNode("Recorded collaboration")),
      $createListNode("number", 3).append(
        $createListItemNode().append($createTextNode("First item")),
        $createListItemNode().append($createTextNode("Second item")),
      ),
    );
  },
  { discrete: true },
);
const bytes = `\\x${Buffer.from(Y.encodeStateAsUpdate(doc)).toString("hex")}`;
stop();
doc.destroy();
const fixture = replayFixture();
fixture.start_ns = "0";
fixture.duration_ns = "12000000000";
fixture.intervals = [
  { generation: 1, start_ns: "0", end_ns: "6000000000" },
  { generation: 2, start_ns: "7000000000", end_ns: "12000000000" },
];
fixture.snapshots = fixture.snapshots.filter((snapshot) => snapshot.generation === 1);
fixture.snapshots.push(
  {
    generation: 2,
    snapshot_kind: "document",
    source_id: "notes",
    payload: { kind: "notes", schema_version: 1 },
  },
  {
    generation: 2,
    snapshot_kind: "checkpoint",
    source_id: "notes",
    payload: { document_id: "notes", update_bytes: bytes },
  },
);
fixture.events = fixture.events.map((event, index) => ({
  ...event,
  at_ns: String((index + 1) * 1000000000),
}));
const presentation = (at: string, payload: Record<string, unknown>) => ({
  generation: 1,
  event_kind: "collaboration",
  source_id: `presentation-${at}`,
  at_ns: at,
  payload: { kind: "presentation", payload },
});
fixture.events.push(
  presentation("1500000000", { action: "select", file_ids: ["image"] }),
  presentation("2500000000", { action: "select", file_ids: ["pdf"] }),
  presentation("3500000000", { action: "select", file_ids: ["video"] }),
  presentation("4000000000", { action: "playback", position_ms: 0, paused: false, rate: 1 }),
  presentation("5500000000", { action: "stop" }),
);
fixture.assets = [
  { file_id: "image", mime: "image/png", archive_key: "image.png" },
  { file_id: "pdf", mime: "application/pdf", archive_key: "deck.pdf" },
  { file_id: "video", mime: "video/webm", archive_key: "camera.webm" },
];
const image = await sharp({
  create: { width: 320, height: 180, channels: 4, background: "#6952e8" },
})
  .png()
  .toBuffer();
await writeFile(join(directory, "image.png"), image);
const content = "BT /F1 24 Tf 30 100 Td (Archived PDF) Tj ET";
const objects = [
  "<< /Type /Catalog /Pages 2 0 R >>",
  "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
  "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 320 180] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>",
  "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
  `<< /Length ${content.length} >>\nstream\n${content}\nendstream`,
];
let pdf = "%PDF-1.4\n";
const offsets = [0];
objects.forEach((object, index) => {
  offsets.push(pdf.length);
  pdf += `${index + 1} 0 obj\n${object}\nendobj\n`;
});
const xref = pdf.length;
pdf += `xref\n0 ${offsets.length}\n0000000000 65535 f \n`;
pdf += offsets
  .slice(1)
  .map((offset) => `${String(offset).padStart(10, "0")} 00000 n \n`)
  .join("");
pdf += `trailer\n<< /Size ${offsets.length} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF`;
await writeFile(join(directory, "deck.pdf"), pdf);
const assetOrigin = process.env.MAHFEL_RECORDING_FIXTURE_ASSET_ORIGIN ?? "http://127.0.0.1:3011";
function renderAsset(path: string, bytes: Buffer) {
  return {
    url: `${assetOrigin}/${path}`,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
}
await writeFile(
  join(directory, "input.json"),
  JSON.stringify({ ...fixture, width: 1920, height: 1080 }),
);
await writeFile(
  join(directory, "media.json"),
  JSON.stringify([
    {
      id: "camera",
      actorId: "mahyar",
      source: "camera",
      path: "camera.webm",
      start_ns: "0",
      end_ns: "12000000000",
      audio: false,
    },
    {
      id: "microphone",
      actorId: "mahyar",
      source: "microphone",
      path: "microphone.mka",
      start_ns: "0",
      end_ns: "12000000000",
      audio: true,
    },
  ]),
);
await writeFile(
  join(directory, "assets.json"),
  JSON.stringify({
    image: { path: "image.png", mime: "image/png", render: renderAsset("image.png", image) },
    pdf: {
      path: "deck.pdf",
      mime: "application/pdf",
      render: renderAsset("deck.pdf", Buffer.from(pdf)),
    },
    video: { path: "camera.webm", mime: "video/webm" },
  }),
);
await writeFile(
  join(directory, "settings.json"),
  JSON.stringify({ output_bytes: 100000000, max_active_audio_tracks: 64 }),
);
