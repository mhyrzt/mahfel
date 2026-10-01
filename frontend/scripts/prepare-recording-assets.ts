import { mkdir, copyFile, cp } from "node:fs/promises";
import { resolve } from "node:path";

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
