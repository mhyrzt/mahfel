import { mkdir, copyFile, cp, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const output = resolve(process.argv[2] ?? "recording-renderer-dist");
await mkdir(output, { recursive: true });
const result = await Bun.build({
  entrypoints: ["features/recordings/renderer/browser.ts"],
  outdir: output,
  target: "browser",
  minify: true,
  define: { "process.env.NODE_ENV": JSON.stringify("production") },
  naming: "renderer.js",
});
if (!result.success) {
  throw new AggregateError(result.logs, "Recording renderer build failed");
}
await copyFile("features/recordings/renderer/renderer.css", `${output}/renderer.css`);
await copyFile("node_modules/pdfjs-dist/build/pdf.worker.min.mjs", `${output}/pdf.worker.mjs`);
await copyFile(
  "node_modules/@fontsource-variable/vazirmatn/files/vazirmatn-arabic-wght-normal.woff2",
  `${output}/font.woff2`,
);
await cp("node_modules/@excalidraw/excalidraw/dist/prod/fonts", `${output}/fonts`, {
  recursive: true,
});
await copyFile("scripts/recording-renderer/runner.ts", `${output}/runner.ts`);
await writeFile(
  `${output}/index.html`,
  '<!doctype html><html><head><meta charset="utf-8"><link rel="stylesheet" href="/renderer.css"></head><body><script>window.EXCALIDRAW_ASSET_PATH="/"</script><script type="module" src="/renderer.js"></script></body></html>',
);

const web = resolve("public/recording-renderer");
await mkdir(web, { recursive: true });
await copyFile("node_modules/pdfjs-dist/build/pdf.worker.min.mjs", `${web}/pdf.worker.mjs`);
await copyFile(
  "node_modules/@fontsource-variable/vazirmatn/files/vazirmatn-arabic-wght-normal.woff2",
  `${web}/font.woff2`,
);
await cp("node_modules/@excalidraw/excalidraw/dist/prod/fonts", `${web}/fonts`, {
  recursive: true,
});
