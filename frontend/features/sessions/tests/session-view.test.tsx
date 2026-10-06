import { act, renderHook } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import {
  parseSessionSurface,
  sessionViewUrl,
  useSessionView,
} from "../components/use-session-view";

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(window.location.search),
}));

afterEach(() => window.history.replaceState(null, "", "/"));

test("validates saved views and retains unrelated query state and selected question", () => {
  expect(parseSessionSurface("whiteboard")).toBe("whiteboard");
  expect(parseSessionSurface("unexpected")).toBe("participants");
  expect(
    sessionViewUrl("https://mahfel.localhost/session/1?question=q2&other=keep#anchor", {
      view: "activities",
    }),
  ).toBe("/session/1?question=q2&other=keep&view=activities#anchor");
});

test("restores URL selection and changes views without losing the selected question", () => {
  window.history.replaceState(null, "", "/session/1?view=activities&question=q2");
  const { result, rerender } = renderHook(() => useSessionView());
  expect(result.current.activeSurface).toBe("activities");
  expect(result.current.selectedActivityId).toBe("q2");
  act(() => result.current.setActiveSurface("whiteboard"));
  rerender();
  expect(result.current.activeSurface).toBe("whiteboard");
  expect(result.current.selectedActivityId).toBe("q2");
  act(() => result.current.selectActivity("q3"));
  expect(window.location.search).toBe("?view=whiteboard&question=q3");
  act(() => window.history.replaceState(null, "", "/session/1?view=activities&question=q2"));
  rerender();
  expect(result.current.activeSurface).toBe("activities");
  expect(result.current.selectedActivityId).toBe("q2");
});
