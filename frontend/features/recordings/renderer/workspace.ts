import { exportToCanvas, getCommonBounds, restoreElements } from "@excalidraw/excalidraw";
import type { BinaryFiles, BinaryFileData } from "@excalidraw/excalidraw/types";
import { getDocument, GlobalWorkerOptions } from "pdfjs-dist";
import { readElements } from "../../whiteboard/board-elements";
import { ReplayState } from "../replay/state";
import { describeScene } from "../replay/scene";
import { nanoseconds, parseReplayInput, type VideoTrack } from "../replay/types";
import { ArchivedNotes } from "./notes";

declare global {
  interface Window {
    EXCALIDRAW_ASSET_PATH: string;
  }
}

export function createWorkspaceRenderer(
  root: HTMLElement,
  assetBase = "",
  resolveAsset?: (id: string) => string,
) {
  root.classList.add("recording-scene");
  const font = new FontFace("RecordingVazirmatn", `url("${assetBase}/font.woff2")`, {
    weight: "100 900",
  });
  document.fonts.add(font);
  window.EXCALIDRAW_ASSET_PATH = `${assetBase}/`;
  GlobalWorkerOptions.workerSrc = `${assetBase}/pdf.worker.mjs`;
  let state: ReplayState;
  let notes: ArchivedNotes | null = null;
  let notesGeneration: number | null = null;
  const workspace = document.createElement("div");
  workspace.id = "workspace";
  const mediaRoot = document.createElement("div");
  mediaRoot.className = "recording-media";
  const notesElement = document.createElement("div");
  const boardCanvas = document.createElement("canvas");
  const toasts = document.createElement("div");
  toasts.id = "toasts";
  const labels = document.createElement("div");
  const warning = document.createElement("div");
  warning.className = "coverage-warning";
  warning.textContent = "Partial recording — some content may be unavailable";
  const slate = document.createElement("div");
  slate.id = "slate";
  workspace.append(notesElement, boardCanvas);
  root.append(workspace, mediaRoot, toasts, labels, slate, warning);
  const assetUrl = (id: string) => resolveAsset?.(id) ?? `/assets/${encodeURIComponent(id)}`;
  const asset = (id: string) => state.input.assets.find((item) => item.file_id === id);
  let lastWorkspaceKey = "";
  let lastSurfaceKey = "";
  let pdf: Awaited<ReturnType<typeof getDocument>["promise"]> | null = null;
  let pdfId: string | null = null;

  async function board(): Promise<void> {
    const doc = state.document("whiteboard");
    if (!doc) {
      return;
    }
    const elements = restoreElements(
      readElements(doc.getMap("elements"), doc.getMap("image-assets")),
      null,
    );
    const files: BinaryFiles = {};
    for (const [id] of doc.getMap<string>("image-assets")) {
      if (!asset(id)) {
        continue;
      }
      const response = await fetch(assetUrl(id));
      if (!response.ok) {
        throw new Error("Archived image unavailable");
      }
      const bytes = new Uint8Array(await response.arrayBuffer());
      const base64 = btoa(Array.from(bytes, (value) => String.fromCharCode(value)).join(""));
      files[id] = {
        id: id as BinaryFileData["id"],
        dataURL:
          `data:${asset(id)?.mime ?? "image/png"};base64,${base64}` as BinaryFileData["dataURL"],
        mimeType: asset(id)?.mime === "image/jpeg" ? "image/jpeg" : "image/png",
        created: 0,
      };
    }
    const ctx = boardCanvas.getContext("2d");
    if (!ctx) {
      throw new Error("Canvas unavailable");
    }
    ctx.clearRect(0, 0, boardCanvas.width, boardCanvas.height);
    if (!elements.length) {
      return;
    }
    const rendered = await exportToCanvas({
      elements,
      files,
      exportPadding: 0,
      appState: { exportBackground: false, exportWithDarkMode: false },
      getDimensions: (width: number, height: number) => ({
        width,
        height,
        scale: Math.min(1, 4096 / Math.max(1, width), 4096 / Math.max(1, height)),
      }),
    });
    const [x, y, maxX, maxY] = getCommonBounds(elements);
    const viewport = state.viewports.get("whiteboard");
    const zoom = viewport
      ? Number(viewport.zoom ?? 1)
      : Math.min(
          boardCanvas.width / Math.max(1, maxX - x),
          boardCanvas.height / Math.max(1, maxY - y),
        );
    const left = viewport
      ? (x + Number(viewport.x ?? 0)) * zoom
      : (boardCanvas.width - (maxX - x) * zoom) / 2;
    const top = viewport
      ? (y + Number(viewport.y ?? 0)) * zoom
      : (boardCanvas.height - (maxY - y) * zoom) / 2;
    ctx.drawImage(rendered, left, top, (maxX - x) * zoom, (maxY - y) * zoom);
  }

  async function presentation(): Promise<void> {
    const current = state.presentation;
    if (!current) {
      return;
    }
    const id =
      current.fileIds.length === 1 ? current.fileIds[0] : current.fileIds[current.page - 1];
    if (!id) {
      throw new Error("Presentation asset missing");
    }
    if (!asset(id)) {
      const ctx = boardCanvas.getContext("2d");
      if (!ctx) {
        throw new Error("Canvas unavailable");
      }
      ctx.clearRect(0, 0, boardCanvas.width, boardCanvas.height);
      ctx.font = "24px sans-serif";
      ctx.fillText("Archived presentation unavailable", 32, 64);
      return;
    }
    if (asset(id)?.mime?.startsWith("video/")) {
      workspace.style.background = "transparent";
      return;
    }
    const ctx = boardCanvas.getContext("2d");
    if (!ctx) {
      throw new Error("Canvas unavailable");
    }
    ctx.clearRect(0, 0, boardCanvas.width, boardCanvas.height);
    if (asset(id)?.mime === "application/pdf") {
      if (pdfId !== id) {
        pdf?.cleanup();
        pdf = await getDocument({ url: assetUrl(id) }).promise;
        pdfId = id;
      }
      if (!pdf) {
        throw new Error("PDF unavailable");
      }
      const page = await pdf.getPage(current.page);
      const base = page.getViewport({ scale: 1 });
      const viewport = page.getViewport({
        scale: Math.min(boardCanvas.width / base.width, boardCanvas.height / base.height),
      });
      const canvas = document.createElement("canvas");
      canvas.width = Math.ceil(viewport.width);
      canvas.height = Math.ceil(viewport.height);
      await page.render({ canvas, viewport }).promise;
      ctx.drawImage(
        canvas,
        (boardCanvas.width - canvas.width) / 2,
        (boardCanvas.height - canvas.height) / 2,
      );
      page.cleanup();
    } else {
      const image = new Image();
      image.src = assetUrl(id);
      await image.decode();
      const scale = Math.min(boardCanvas.width / image.width, boardCanvas.height / image.height);
      ctx.drawImage(
        image,
        (boardCanvas.width - image.width * scale) / 2,
        (boardCanvas.height - image.height * scale) / 2,
        image.width * scale,
        image.height * scale,
      );
    }
  }

  async function initialize(value: unknown): Promise<void> {
    state?.destroy();
    notes?.destroy();
    notes = null;
    notesGeneration = null;
    lastWorkspaceKey = "";
    lastSurfaceKey = "";
    state = new ReplayState(parseReplayInput(value));
    await font.load();
    await document.fonts.ready;
  }

  async function render(timestamp: string, tracks: VideoTrack[], width: number, height: number) {
    const atNs = nanoseconds(timestamp);
    state.advance(atNs);
    const scene = describeScene(state, atNs, tracks, width, height);
    warning.style.display = state.input.partial ? "block" : "none";
    root.style.width = `${width}px`;
    root.style.height = `${height}px`;
    workspace.style.width = `${width}px`;
    workspace.style.height = `${(height * 5) / 6}px`;
    workspace.style.display = scene.surface && !scene.slate ? "block" : "none";
    const selectedPresentation = state.presentation;
    const selectedFile =
      selectedPresentation?.fileIds[
        selectedPresentation.fileIds.length === 1 ? 0 : selectedPresentation.page - 1
      ];
    workspace.style.background =
      scene.surface === "presentation" &&
      selectedFile &&
      asset(selectedFile)?.mime?.startsWith("video/")
        ? "transparent"
        : "#fafafc";
    notesElement.style.display = scene.surface === "notes" ? "block" : "none";
    boardCanvas.style.display =
      scene.surface === "whiteboard" || scene.surface === "presentation" ? "block" : "none";
    if (boardCanvas.width !== width) {
      boardCanvas.width = width;
    }
    if (boardCanvas.height !== (height * 5) / 6) {
      boardCanvas.height = (height * 5) / 6;
    }
    const surfaceKey = JSON.stringify({
      surface: scene.surface,
      generation: state.generation,
      width,
      height,
      documents: [...state.documents.values()].map((item) => item.revision),
      presentation: state.presentation
        ? { ...state.presentation, anchorNs: state.presentation.anchorNs.toString() }
        : null,
      viewports: [...state.viewports],
    });
    const surfaceChanged = surfaceKey !== lastSurfaceKey;
    lastSurfaceKey = surfaceKey;
    if (scene.surface === "notes") {
      if (notesGeneration !== state.generation) {
        notes?.destroy();
        notesElement.replaceChildren();
        notes = new ArchivedNotes(notesElement, (id) => (asset(id) ? assetUrl(id) : ""));
        notesGeneration = state.generation;
      }
      const doc = state.document("notes");
      if (doc) {
        notes?.update(doc);
      }
      notesElement.scrollTop = Number(state.viewports.get("notes")?.y ?? 0);
    } else if (scene.surface === "whiteboard" && surfaceChanged) {
      await board();
    } else if (scene.surface === "presentation" && surfaceChanged) {
      await presentation();
    }
    toasts.style.bottom = `${height / 6 + 24}px`;
    toasts.replaceChildren(
      ...state.toasts.map((toast) => {
        const node = document.createElement("article");
        node.dir = "auto";
        const author = document.createElement("strong");
        author.textContent = toast.author;
        const body = document.createElement("p");
        body.textContent = toast.body;
        node.append(author, body);
        return node;
      }),
    );
    labels.replaceChildren(
      ...scene.layers
        .filter((layer) => layer.source !== "workspace")
        .map((layer) => {
          const name = document.createElement("span");
          const track = tracks.find((item) => item.id === layer.source);
          name.textContent = track ? (state.input.actors[track.actorId] ?? "Participant") : "";
          name.style.cssText = `position:absolute;left:${layer.x + 12}px;top:${layer.y + layer.height - 38}px;max-width:${layer.width - 24}px`;
          name.className = "camera-label";
          name.dir = "auto";
          return name;
        }),
    );
    slate.style.display = scene.slate ? "grid" : "none";
    slate.textContent = scene.slate === "paused" ? "Recording paused" : "Recording unavailable";
    const current = state.presentation;
    const fileId = current?.fileIds[current.fileIds.length === 1 ? 0 : current.page - 1];
    const presentationVideo =
      scene.surface === "presentation" && fileId && asset(fileId)?.mime?.startsWith("video/")
        ? {
            file_id: fileId,
            position_ms: state.presentationPosition(nanoseconds(state.input.start_ns) + atNs),
            paused: current?.paused ?? true,
            rate: current?.rate ?? 1,
          }
        : null;
    await new Promise<void>((resolve) =>
      requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
    );
    if (scene.surface === "notes") {
      notesElement.scrollTop = Number(state.viewports.get("notes")?.y ?? 0);
    }
    await Promise.all(
      Array.from(notesElement.querySelectorAll("img")).map((image) => image.decode()),
    );
    await document.fonts.ready;
    const key = JSON.stringify({
      scene: { ...scene, timestamp_ns: "" },
      generation: state.generation,
      toasts: state.toasts.map((item) => ({ id: item.id, body: item.body })),
      documents: [...state.documents.values()].map((item) => item.revision),
      presentation: state.presentation
        ? { ...state.presentation, anchorNs: state.presentation.anchorNs.toString() }
        : null,
      viewports: [...state.viewports],
    });
    const changed = key !== lastWorkspaceKey;
    lastWorkspaceKey = key;
    return { scene, presentationVideo, changed };
  }

  return {
    mediaRoot,
    initialize,
    render,
    destroy() {
      state?.destroy();
      notes?.destroy();
      pdf?.cleanup();
      document.fonts.delete(font);
      root.replaceChildren();
    },
  };
}
