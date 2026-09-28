import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";

vi.mock("next/dynamic", () => ({
  default:
    () =>
    ({ viewModeEnabled }: { viewModeEnabled: boolean }) => (
      <button disabled={viewModeEnabled}>Whiteboard canvas</button>
    ),
}));
vi.mock("@excalidraw/excalidraw", () => ({
  restoreElements: (elements: unknown) => elements,
  CaptureUpdateAction: { NEVER: "never" },
}));
vi.mock("@/lib/collaboration/provider", () => ({
  MahfelCollaborationProvider: class {
    awareness = {
      setLocalStateField: vi.fn(),
      getStates: () => new Map(),
      on: vi.fn(),
      off: vi.fn(),
    };
    on = vi.fn();
    off = vi.fn();
    connect = vi.fn();
    destroy = vi.fn();
  },
}));

import { WhiteboardEditor } from "../components/whiteboard-editor";

test("a live manager can draw before the collaboration socket finishes connecting", () => {
  render(
    <WhiteboardEditor canEdit live locale="en" onEvent={() => undefined} sessionId="session-1" />,
  );

  expect(screen.getByRole("button", { name: "Whiteboard canvas" })).toBeEnabled();
});

test("a viewer cannot draw on the whiteboard", () => {
  render(
    <WhiteboardEditor
      canEdit={false}
      live
      locale="en"
      onEvent={() => undefined}
      sessionId="session-1"
    />,
  );

  expect(screen.getByRole("button", { name: "Whiteboard canvas" })).toBeDisabled();
});
