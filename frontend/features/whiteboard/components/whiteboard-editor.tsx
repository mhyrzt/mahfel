"use client";

import "@excalidraw/excalidraw/index.css";

import { restoreElements, CaptureUpdateAction } from "@excalidraw/excalidraw";
import type { ExcalidrawImperativeAPI, NormalizedZoomValue } from "@excalidraw/excalidraw/types";
import type { ExcalidrawElement } from "@excalidraw/excalidraw/element/types";
import dynamic from "next/dynamic";
import { useEffect, useMemo, useRef, useState } from "react";
import * as Y from "yjs";

import { browserApiClient } from "@/lib/api/browser";
import type { Locale } from "@/lib/i18n/config";
import { messagesFor } from "@/lib/i18n/messages";
import {
  MahfelCollaborationProvider,
  type CollaborationEventFrame,
} from "@/lib/collaboration/provider";

const Excalidraw = dynamic(
  () => import("@excalidraw/excalidraw").then((module) => module.Excalidraw),
  {
    ssr: false,
  },
);
const supportedElementTypes = new Set([
  "rectangle",
  "ellipse",
  "diamond",
  "arrow",
  "line",
  "freedraw",
  "text",
  "frame",
]);

interface WhiteboardEditorProps {
  sessionId: string;
  canEdit: boolean;
  live: boolean;
  locale: Locale;
  onEvent: (event: CollaborationEventFrame) => void;
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
  sessionId,
  canEdit: initialCanEdit,
  live,
  locale,
  onEvent,
}: Readonly<WhiteboardEditorProps>) {
  const text = messagesFor(locale).collaboration;
  const [canEdit, setCanEdit] = useState(initialCanEdit);
  const [status, setStatus] = useState("connecting");
  const api = useRef<ExcalidrawImperativeAPI | null>(null);
  const applyingRemote = useRef(false);
  const latestViewport = useRef<{ x: number; y: number; zoom: number } | null>(null);
  const lastViewport = useRef(0);
  const doc = useMemo(() => new Y.Doc({ guid: sessionId }), [sessionId]);
  const elements = useMemo(() => doc.getMap<string>("elements"), [doc]);
  const provider = useMemo(
    () =>
      new MahfelCollaborationProvider(
        sessionId,
        "whiteboard",
        doc,
        setCanEdit,
        // eslint-disable-next-line react-hooks/refs
        (event) => receiveBoardEvent(event, api, applyingRemote, latestViewport, onEvent),
      ),
    [sessionId, doc, onEvent],
  );

  useEffect(() => {
    const onStatus = ({ status }: { status: string }) => setStatus(status);
    const render = () => {
      if (!api.current) {
        return;
      }
      applyingRemote.current = true;
      const scene = readElements(elements);
      api.current.updateScene({
        elements: restoreElements(scene, null, { repairBindings: true }),
        captureUpdate: CaptureUpdateAction.NEVER,
      });
      window.setTimeout(() => {
        applyingRemote.current = false;
      }, 0);
    };
    provider.on("status", onStatus);
    elements.observe(render);
    provider.connect();
    return () => {
      provider.off("status", onStatus);
      elements.unobserve(render);
      provider.destroy();
      doc.destroy();
    };
  }, [doc, elements, provider]);

  const onChange: NonNullable<React.ComponentProps<typeof Excalidraw>["onChange"]> = () => {
    if (!canEdit || applyingRemote.current || !api.current) {
      return;
    }
    const current = api.current.getSceneElementsIncludingDeleted();
    doc.transact(() => {
      for (const element of current) {
        if (!supportedElementTypes.has(element.type)) {
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
    const scene = readElements(elements);
    if (scene.length) {
      value.updateScene({
        elements: restoreElements(scene, null, { repairBindings: true }),
        captureUpdate: CaptureUpdateAction.NEVER,
      });
    }
    if (latestViewport.current) {
      applyViewport(value, applyingRemote, latestViewport.current);
    }
  };

  const onScrollChange = (x: number, y: number, zoom: { value: number }) => {
    if (!live || applyingRemote.current) {
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

  return (
    <div className="overflow-hidden rounded-lg border bg-background">
      <p className="border-b px-3 py-2 text-xs text-muted-foreground" role="status">
        {status === "connected" ? text.connected : text.loading}
      </p>
      <div className="h-[36rem]">
        <Excalidraw
          excalidrawAPI={onApi}
          onChange={onChange}
          onScrollChange={onScrollChange}
          viewModeEnabled={!canEdit}
          UIOptions={{
            canvasActions: { loadScene: false, saveToActiveFile: false, export: false },
          }}
        />
      </div>
    </div>
  );
}

function readElements(elements: Y.Map<string>): ExcalidrawElement[] {
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
