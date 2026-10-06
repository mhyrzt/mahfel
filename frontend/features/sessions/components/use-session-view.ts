"use client";

import { useCallback } from "react";
import { useSearchParams } from "next/navigation";

export const sessionSurfaces = [
  "participants",
  "notes",
  "whiteboard",
  "presentation",
  "activities",
] as const;
export type SessionSurface = (typeof sessionSurfaces)[number];

export function parseSessionSurface(value: string | null): SessionSurface {
  return sessionSurfaces.find((surface) => surface === value) ?? "participants";
}

export function sessionViewUrl(
  url: string,
  changes: { view?: SessionSurface; question?: string },
): string {
  const next = new URL(url);
  if (changes.view) {
    next.searchParams.set("view", changes.view);
  }
  if (changes.question) {
    next.searchParams.set("question", changes.question);
  }
  return `${next.pathname}${next.search}${next.hash}`;
}

export function useSessionView() {
  const params = useSearchParams();
  const activeSurface = parseSessionSurface(params.get("view"));
  const selectedActivityId = params.get("question");
  const update = useCallback((changes: { view?: SessionSurface; question?: string }) => {
    const next = sessionViewUrl(window.location.href, changes);
    if (next !== `${window.location.pathname}${window.location.search}${window.location.hash}`) {
      window.history.pushState(null, "", next);
    }
  }, []);
  const setActiveSurface = useCallback((view: SessionSurface) => update({ view }), [update]);
  const selectActivity = useCallback((question: string) => update({ question }), [update]);
  return { activeSurface, setActiveSurface, selectedActivityId, selectActivity };
}
