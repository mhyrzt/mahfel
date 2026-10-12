"use client";

import { useEffect, useRef, useState } from "react";
import type { PartoController, PartoOptions } from "../core/renderer";

export interface PartoProps extends PartoOptions {
  label?: string;
  className?: string;
  sequence?: "confused" | "delight";
  interactive?: boolean;
  followCursor?: boolean;
  interactionLabel?: string;
  errorLabel?: string;
  onError?: (error: Error) => void;
}
export function Parto({
  label = "Parto — Mahfel assistant",
  className,
  onError,
  sequence,
  interactive = false,
  followCursor = false,
  interactionLabel,
  errorLabel = "Parto renderer unavailable",
  ...options
}: PartoProps) {
  const host = useRef<HTMLDivElement>(null);
  const controller = useRef<PartoController | null>(null);
  const latest = useRef({ options, sequence, onError });
  const [error, setError] = useState(false);
  useEffect(() => {
    latest.current = { options, sequence, onError };
    controller.current?.update(options);
  }, [options, sequence, onError]);
  useEffect(() => {
    let disposed = false;
    const element = host.current;
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateMotion = () =>
      controller.current?.update({
        reducedMotion: latest.current.options.reducedMotion ?? media.matches,
      });
    media.addEventListener("change", updateMotion);
    if (element) {
      void import("../core/renderer")
        .then(({ createParto }) =>
          createParto(element, {
            ...latest.current.options,
            reducedMotion: latest.current.options.reducedMotion ?? media.matches,
          }),
        )
        .then((instance) => {
          if (disposed) {
            instance.destroy();
            return;
          }
          controller.current = instance;
          instance.update(latest.current.options);
          updateMotion();
          if (latest.current.sequence) {
            instance.play(latest.current.sequence);
          }
        })
        .catch((reason: unknown) => {
          if (!disposed) {
            setError(true);
            latest.current.onError?.(reason instanceof Error ? reason : new Error(String(reason)));
          }
        });
    }
    return () => {
      disposed = true;
      media.removeEventListener("change", updateMotion);
      controller.current?.destroy();
      controller.current = null;
    };
  }, []);
  useEffect(() => {
    if (sequence) {
      controller.current?.play(sequence);
    }
  }, [sequence]);
  useEffect(() => {
    if (!followCursor) {
      return undefined;
    }
    const move = (event: PointerEvent) => {
      if (event.pointerType === "touch") {
        return;
      }
      const bounds = host.current?.getBoundingClientRect();
      if (!bounds || bounds.width === 0 || bounds.height === 0) {
        return;
      }
      controller.current?.update({
        gazeX: (event.clientX - bounds.left - bounds.width / 2) / (bounds.width * 0.75),
        gazeY: (event.clientY - bounds.top - bounds.height / 2) / (bounds.height * 0.75),
      });
    };
    const reset = () =>
      controller.current?.update({
        gazeX: latest.current.options.gazeX ?? 0,
        gazeY: latest.current.options.gazeY ?? 0,
      });
    window.addEventListener("pointermove", move, { passive: true });
    window.addEventListener("blur", reset);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("blur", reset);
      reset();
    };
  }, [followCursor]);
  return (
    <div
      className={className}
      role={interactive ? "button" : "img"}
      tabIndex={interactive ? 0 : undefined}
      onPointerMove={
        interactive
          ? (event) => {
              const bounds = event.currentTarget.getBoundingClientRect();
              controller.current?.update({
                gazeX: ((event.clientX - bounds.left) / bounds.width) * 2 - 1,
                gazeY: ((event.clientY - bounds.top) / bounds.height) * 2 - 1,
              });
            }
          : undefined
      }
      onPointerLeave={
        interactive
          ? () =>
              controller.current?.update({ gazeX: options.gazeX ?? 0, gazeY: options.gazeY ?? 0 })
          : undefined
      }
      onClick={interactive ? () => controller.current?.play("delight") : undefined}
      onKeyDown={
        interactive
          ? (event) => {
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                controller.current?.play("delight");
              }
            }
          : undefined
      }
      aria-label={
        interactive ? (interactionLabel ?? `${label}. Activate for a playful hop`) : label
      }
      style={{
        width: options.size ?? 256,
        height: options.size ?? 256,
        cursor: interactive ? "pointer" : undefined,
        borderRadius: 16,
      }}
    >
      <div ref={host} />
      {error && <span role="alert">{errorLabel}</span>}
    </div>
  );
}
