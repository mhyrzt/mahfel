import type { ExcalidrawElement } from "@excalidraw/excalidraw/element/types";
import type * as Y from "yjs";

export const supportedElementTypes = new Set([
  "rectangle",
  "ellipse",
  "diamond",
  "arrow",
  "line",
  "freedraw",
  "text",
  "frame",
  "image",
]);

export function readElements(elements: Y.Map<string>, assets: Y.Map<string>): ExcalidrawElement[] {
  return Array.from(elements.values())
    .flatMap((serialized) => {
      try {
        const value: unknown = JSON.parse(serialized);
        if (
          typeof value === "object" &&
          value !== null &&
          "id" in value &&
          typeof value.id === "string" &&
          "type" in value &&
          typeof value.type === "string" &&
          supportedElementTypes.has(value.type) &&
          "x" in value &&
          typeof value.x === "number" &&
          Number.isFinite(value.x) &&
          "y" in value &&
          typeof value.y === "number" &&
          Number.isFinite(value.y)
        ) {
          if (
            value.type === "image" &&
            (!("fileId" in value) || typeof value.fileId !== "string" || !assets.has(value.fileId))
          ) {
            return [];
          }
          return [value as ExcalidrawElement];
        }
      } catch {}
      return [];
    })
    .sort((left, right) => {
      const order = (left.index ?? "").localeCompare(right.index ?? "");
      return order || left.id.localeCompare(right.id);
    });
}
