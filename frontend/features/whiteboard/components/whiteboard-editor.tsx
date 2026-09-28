"use client";

import "@excalidraw/excalidraw/index.css";

import { restoreElements, CaptureUpdateAction } from "@excalidraw/excalidraw";
import type {
  BinaryFiles,
  Collaborator,
  ExcalidrawImperativeAPI,
  NormalizedZoomValue,
  SocketId,
} from "@excalidraw/excalidraw/types";
import dynamic from "next/dynamic";
import { useEffect, useMemo, useRef, useState } from "react";
import * as Y from "yjs";

import { browserApiClient } from "@/lib/api/browser";
import {
  excalidrawImageData,
  isSupportedCollaborationImage,
  registerCollaborationImage,
} from "@/lib/collaboration/images";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import {
  MahfelCollaborationProvider,
  type CollaborationEventFrame,
} from "@/lib/collaboration/provider";

import { readElements, supportedElementTypes } from "../board-elements";

const Excalidraw = dynamic(
  () => import("@excalidraw/excalidraw").then((module) => module.Excalidraw),
  {
    ssr: false,
  },
);

interface WhiteboardEditorProps {
  displayName?: string;
  sessionId: string;
  canEdit: boolean;
  live: boolean;
  locale: Locale;
  onEvent: (event: CollaborationEventFrame) => void;
}

type WhiteboardStatus =
  "connecting" | "connected" | "reconnecting" | "unauthorized" | "closed" | "image-error";

function isConnectionStatus(status: string): status is Exclude<WhiteboardStatus, "image-error"> {
  return ["connecting", "connected", "reconnecting", "unauthorized", "closed"].includes(status);
}

function receiveBoardEvent(
  event: CollaborationEventFrame,
  api: React.RefObject<ExcalidrawImperativeAPI | null>,
  applyingRemote: React.RefObject<boolean>,
  latestViewport: React.RefObject<{ x: number; y: number; zoom: number } | null>,
  onEvent: (event: CollaborationEventFrame) => void,
) {
  if (event.kind === "viewport" && event.data.surface === "whiteboard") {
    const { x, y, zoom } = event.data;
    if (typeof x === "number" && typeof y === "number" && typeof zoom === "number") {
      latestViewport.current = { x, y, zoom };
      if (api.current) {
        applyViewport(api.current, applyingRemote, latestViewport.current);
      }
    }
  }
  onEvent(event);
}

function applyViewport(
  api: ExcalidrawImperativeAPI,
  applyingRemote: React.RefObject<boolean>,
  viewport: { x: number; y: number; zoom: number },
) {
  applyingRemote.current = true;
  api.updateScene({
    appState: {
      scrollX: viewport.x,
      scrollY: viewport.y,
      zoom: { value: viewport.zoom as NormalizedZoomValue },
    },
  });
  window.setTimeout(() => {
    applyingRemote.current = false;
  }, 150);
}

export function WhiteboardEditor({
  displayName,
  sessionId,
  canEdit: grantedEdit,
  live,
  locale,
  onEvent,
}: Readonly<WhiteboardEditorProps>) {
  const text = messagesFor(locale).collaboration;
  const [socketCanEdit, setSocketCanEdit] = useState<boolean | null>(null);
  const canEdit = live && (socketCanEdit ?? grantedEdit);
  const [status, setStatus] = useState<WhiteboardStatus>("connecting");
  const api = useRef<ExcalidrawImperativeAPI | null>(null);
  const applyingRemote = useRef(false);
  const latestViewport = useRef<{ x: number; y: number; zoom: number } | null>(null);
  const lastViewport = useRef(0);
  const uploading = useRef(new Set<string>());
  const loadingImages = useRef(new Set<string>());
  const doc = useMemo(() => new Y.Doc({ guid: sessionId }), [sessionId]);
  const elements = useMemo(() => doc.getMap<string>("elements"), [doc]);
  const assets = useMemo(() => doc.getMap<string>("image-assets"), [doc]);
  const provider = useMemo(
    () =>
      new MahfelCollaborationProvider(
        sessionId,
        "whiteboard",
        doc,
        setSocketCanEdit,
        // eslint-disable-next-line react-hooks/refs
        (event) => receiveBoardEvent(event, api, applyingRemote, latestViewport, onEvent),
      ),
    [sessionId, doc, onEvent],
  );

  useEffect(() => {
    provider.awareness.setLocalStateField("user", { name: displayName ?? "Participant" });
    const updateCollaborators = () => {
      const collaborators = new Map<SocketId, Collaborator>();
      for (const [clientId, state] of provider.awareness.getStates()) {
        if (clientId === doc.clientID || typeof state !== "object" || state === null) {
          continue;
        }
        const user = "user" in state ? state.user : null;
        const pointer = "pointer" in state ? state.pointer : null;
        if (
          typeof pointer !== "object" ||
          pointer === null ||
          !("x" in pointer) ||
          !("y" in pointer) ||
          typeof pointer.x !== "number" ||
          typeof pointer.y !== "number"
        ) {
          continue;
        }
        const name =
          typeof user === "object" &&
          user !== null &&
          "name" in user &&
          typeof user.name === "string"
            ? user.name
            : "Participant";
        collaborators.set(String(clientId) as SocketId, {
          username: name,
          pointer: { x: pointer.x, y: pointer.y, tool: "pointer" },
          color: { background: "#2563eb", stroke: "#1d4ed8" },
        });
      }
      api.current?.updateScene({ collaborators });
    };
    provider.awareness.on("update", updateCollaborators);
    return () => provider.awareness.off("update", updateCollaborators);
  }, [displayName, doc, provider]);

  useEffect(() => {
    const onStatus = ({ status }: { status: string }) => {
      if (isConnectionStatus(status)) {
        setStatus(status);
      }
    };
    const render = (_event: Y.YMapEvent<string>, transaction: Y.Transaction) => {
      if (transaction.local) {
        return;
      }
      if (!api.current) {
        return;
      }
      applyingRemote.current = true;
      const scene = readElements(elements, assets);
      const pending = api.current
        .getSceneElementsIncludingDeleted()
        .filter(
          (element) =>
            element.type === "image" &&
            element.fileId !== null &&
            uploading.current.has(element.fileId) &&
            !scene.some((saved) => saved.id === element.id),
        );
      api.current.updateScene({
        elements: restoreElements([...scene, ...pending], null, { repairBindings: true }),
        captureUpdate: CaptureUpdateAction.NEVER,
      });
      for (const element of scene) {
        if (
          element.type !== "image" ||
          !element.fileId ||
          api.current.getFiles()[element.fileId] ||
          loadingImages.current.has(element.fileId)
        ) {
          continue;
        }
        const fileId = assets.get(element.fileId);
        if (!fileId) {
          continue;
        }
        loadingImages.current.add(element.fileId);
        void excalidrawImageData(fileId, element.fileId)
          .then((file) => {
            api.current?.addFiles([file]);
          })
          .catch(() => setStatus("image-error"))
          .finally(() => loadingImages.current.delete(element.fileId ?? ""));
      }
      window.setTimeout(() => {
        applyingRemote.current = false;
      }, 0);
    };
    provider.on("status", onStatus);
    elements.observe(render);
    assets.observe(render);
    provider.connect();
    return () => {
      provider.off("status", onStatus);
      elements.unobserve(render);
      assets.unobserve(render);
      provider.destroy();
      doc.destroy();
    };
  }, [assets, doc, elements, provider]);

  const onChange: NonNullable<React.ComponentProps<typeof Excalidraw>["onChange"]> = (
    _scene,
    _state,
    files: BinaryFiles,
  ) => {
    if (!canEdit || applyingRemote.current || !api.current) {
      return;
    }
    const current = api.current.getSceneElementsIncludingDeleted();
    doc.transact(() => {
      for (const element of current) {
        if (!supportedElementTypes.has(element.type)) {
          continue;
        }
        if (element.type === "image" && element.fileId && !assets.has(element.fileId)) {
          const binary = files[element.fileId];
          if (binary && !uploading.current.has(element.fileId)) {
            const imageId = element.fileId;
            if (!isSupportedCollaborationImage(binary.mimeType)) {
              setStatus("image-error");
              continue;
            }
            uploading.current.add(imageId);
            void fetch(binary.dataURL)
              .then((response) => response.blob())
              .then((blob) =>
                registerCollaborationImage(
                  new File(
                    [blob],
                    `whiteboard-${imageId}.${binary.mimeType === "image/png" ? "png" : "jpg"}`,
                    { type: binary.mimeType },
                  ),
                  sessionId,
                  "whiteboard",
                ),
              )
              .then((fileId) => {
                const latest = api.current
                  ?.getSceneElementsIncludingDeleted()
                  .find((candidate) => candidate.id === element.id);
                if (!latest || latest.isDeleted) {
                  return;
                }
                doc.transact(() => {
                  assets.set(imageId, fileId);
                  elements.set(latest.id, JSON.stringify(latest));
                });
              })
              .catch(() => setStatus("image-error"))
              .finally(() => uploading.current.delete(imageId));
          }
          continue;
        }
        const serialized = JSON.stringify(element);
        if (elements.get(element.id) !== serialized) {
          elements.set(element.id, serialized);
        }
      }
    });
  };

  const onApi = (value: ExcalidrawImperativeAPI) => {
    api.current = value;
    const scene = readElements(elements, assets);
    if (scene.length) {
      value.updateScene({
        elements: restoreElements(scene, null, { repairBindings: true }),
        captureUpdate: CaptureUpdateAction.NEVER,
      });
    }
    if (latestViewport.current) {
      applyViewport(value, applyingRemote, latestViewport.current);
    }
    for (const element of scene) {
      if (element.type !== "image" || !element.fileId) {
        continue;
      }
      const fileId = assets.get(element.fileId);
      if (fileId && !value.getFiles()[element.fileId]) {
        void excalidrawImageData(fileId, element.fileId)
          .then((file) => value.addFiles([file]))
          .catch(() => setStatus("image-error"));
      }
    }
  };

  const onScrollChange = (x: number, y: number, zoom: { value: number }) => {
    if (!live || !canEdit || applyingRemote.current) {
      return;
    }
    const now = Date.now();
    if (now - lastViewport.current < 150) {
      return;
    }
    lastViewport.current = now;
    void browserApiClient.post(
      `/v1/sessions/${encodeURIComponent(sessionId)}/collaboration/viewport`,
      {
        surface: "whiteboard",
        x,
        y,
        zoom: zoom.value,
      },
    );
  };

  let statusText = text.loading;
  if (status === "connected") {
    statusText = text.connected;
  } else if (
    status === "reconnecting" ||
    status === "unauthorized" ||
    status === "closed" ||
    status === "image-error"
  ) {
    statusText = text.unavailable;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border bg-background">
      <div className="flex items-center justify-between gap-3 border-b px-3 py-2 text-xs text-muted-foreground">
        <p role="status">{statusText}</p>
        <span>{canEdit ? text.canEdit : text.viewOnly}</span>
      </div>
      <div
        dir="ltr"
        className={`min-h-0 flex-1 ${canEdit ? "" : "pointer-events-none"}`}
        inert={!canEdit}
      >
        <Excalidraw
          excalidrawAPI={onApi}
          onChange={onChange}
          onScrollChange={onScrollChange}
          onPointerUpdate={({ pointer }) => {
            provider.awareness.setLocalStateField("pointer", pointer);
          }}
          isCollaborating
          viewModeEnabled={!canEdit}
          UIOptions={{
            canvasActions: { loadScene: false, saveToActiveFile: false, export: false },
          }}
        />
      </div>
    </div>
  );
}
