"use client";

import { useEffect, useRef, useState } from "react";
import type { RenderTask } from "pdfjs-dist";

import { fitScale } from "./presentation-fit";

const MAX_RENDER_SCALE = 4;

export function PdfPage({
  bytes,
  page,
  unavailable,
}: Readonly<{ bytes: Uint8Array; page: number; unavailable: string }>) {
  const container = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const [size, setSize] = useState<{ width: number; height: number } | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    const target = container.current;
    if (!target || typeof ResizeObserver === "undefined") {
      return undefined;
    }
    const observer = new ResizeObserver((entries) => {
      const rect = entries[0]?.contentRect;
      if (rect) {
        setSize({ width: rect.width, height: rect.height });
      }
    });
    observer.observe(target);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!size) {
      return undefined;
    }
    let cancelled = false;
    let loading: ReturnType<(typeof import("pdfjs-dist"))["getDocument"]> | null = null;
    let render: RenderTask | null = null;
    void (async () => {
      try {
        const pdfjs = await import("pdfjs-dist");
        pdfjs.GlobalWorkerOptions.workerSrc = new URL(
          "pdfjs-dist/build/pdf.worker.min.mjs",
          import.meta.url,
        ).toString();
        const task = pdfjs.getDocument({ data: bytes.slice() });
        loading = task;
        const pdf = await task.promise;
        if (cancelled) {
          await task.destroy();
          return;
        }
        const sheet = await pdf.getPage(page);
        const base = sheet.getViewport({ scale: 1 });
        const dpr = typeof window === "undefined" ? 1 : window.devicePixelRatio || 1;
        const scale = Math.min(
          fitScale(size.width, size.height, base.width, base.height) * dpr,
          MAX_RENDER_SCALE,
        );
        const viewport = sheet.getViewport({ scale });
        const target = canvas.current;
        if (!target || cancelled) {
          await task.destroy();
          return;
        }
        target.width = Math.ceil(viewport.width);
        target.height = Math.ceil(viewport.height);
        target.style.width = `${viewport.width / dpr}px`;
        target.style.height = `${viewport.height / dpr}px`;
        const context = target.getContext("2d");
        if (!context) {
          await task.destroy();
          return;
        }
        render = sheet.render({ canvas: target, canvasContext: context, viewport });
        await render.promise;
      } catch {
        if (!cancelled) {
          setFailed(true);
        }
      }
    })();
    return () => {
      cancelled = true;
      render?.cancel();
      if (loading) {
        void loading.destroy();
      }
    };
  }, [bytes, page, size]);

  return (
    <div className="flex size-full items-center justify-center" ref={container}>
      {failed ? <p role="alert">{unavailable}</p> : <canvas className="block" ref={canvas} />}
    </div>
  );
}
