"use client";

import { useEffect, useRef, useState } from "react";
import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";

import { fitScale } from "./presentation-fit";

const MAX_RENDER_SCALE = 4;

export function PdfPage({
  url,
  headers,
  page,
  unavailable,
}: Readonly<{ url: string; headers: Record<string, string>; page: number; unavailable: string }>) {
  const container = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const [size, setSize] = useState<{ width: number; height: number } | null>(null);
  const [failed, setFailed] = useState(false);
  const [document, setDocument] = useState<{ url: string; pdf: PDFDocumentProxy } | null>(null);

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
    let cancelled = false;
    let loading: ReturnType<(typeof import("pdfjs-dist"))["getDocument"]> | null = null;
    void (async () => {
      try {
        const pdfjs = await import("pdfjs-dist");
        if (cancelled) {
          return;
        }
        pdfjs.GlobalWorkerOptions.workerSrc = new URL(
          "pdfjs-dist/build/pdf.worker.min.mjs",
          import.meta.url,
        ).toString();
        loading = pdfjs.getDocument({ url, httpHeaders: headers, withCredentials: false });
        const pdf = await loading.promise;
        if (!cancelled) {
          setDocument({ url, pdf });
          setFailed(false);
        }
      } catch {
        if (!cancelled) {
          setFailed(true);
        }
      }
    })();
    return () => {
      cancelled = true;
      if (loading) {
        void loading.destroy();
      }
    };
  }, [url, headers]);

  useEffect(() => {
    if (!size || !document || document.url !== url) {
      return undefined;
    }
    let cancelled = false;
    let render: RenderTask | null = null;
    void (async () => {
      try {
        const sheet = await document.pdf.getPage(page);
        if (cancelled) {
          return;
        }
        const base = sheet.getViewport({ scale: 1 });
        const dpr = window.devicePixelRatio || 1;
        const scale = Math.min(
          fitScale(size.width, size.height, base.width, base.height) * dpr,
          MAX_RENDER_SCALE,
        );
        const viewport = sheet.getViewport({ scale });
        const target = canvas.current;
        if (!target) {
          return;
        }
        target.width = Math.ceil(viewport.width);
        target.height = Math.ceil(viewport.height);
        target.style.width = `${viewport.width / dpr}px`;
        target.style.height = `${viewport.height / dpr}px`;
        const context = target.getContext("2d");
        if (!context) {
          throw new Error("Canvas unavailable");
        }
        render = sheet.render({ canvas: target, canvasContext: context, viewport });
        await render.promise;
        if (!cancelled) {
          setFailed(false);
        }
      } catch {
        if (!cancelled) {
          setFailed(true);
        }
      }
    })();
    return () => {
      cancelled = true;
      render?.cancel();
    };
  }, [document, page, size, url]);

  return (
    <div className="flex size-full items-center justify-center" ref={container}>
      {failed ? <p role="alert">{unavailable}</p> : <canvas className="block" ref={canvas} />}
    </div>
  );
}
