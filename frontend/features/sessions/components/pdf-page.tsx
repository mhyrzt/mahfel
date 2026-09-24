"use client";

import { useEffect, useRef, useState } from "react";
import type { RenderTask } from "pdfjs-dist";

export function PdfPage({
  bytes,
  page,
  unavailable,
}: Readonly<{ bytes: Uint8Array; page: number; unavailable: string }>) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
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
        const viewport = sheet.getViewport({ scale: 1.5 });
        const target = canvas.current;
        if (!target || cancelled) {
          await task.destroy();
          return;
        }
        target.width = Math.ceil(viewport.width);
        target.height = Math.ceil(viewport.height);
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
  }, [bytes, page]);

  return failed ? (
    <p role="alert">{unavailable}</p>
  ) : (
    <canvas ref={canvas} className="mx-auto max-w-full" />
  );
}
