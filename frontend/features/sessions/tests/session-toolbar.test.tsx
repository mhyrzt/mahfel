import { fireEvent, render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";

import { SessionToolbar } from "../components/session-toolbar";

test("media controls stay disabled until a real RTC handler is connected", () => {
  render(<SessionToolbar locale="en" />);
  expect(screen.getByRole("button", { name: "Unmute microphone" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Turn camera on" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Share screen" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Leave session" })).toBeEnabled();
});

test("collaboration surfaces switch from the control toolbar", () => {
  const onSelectSurface = vi.fn();
  render(
    <SessionToolbar locale="en" activeSurface="whiteboard" onSelectSurface={onSelectSurface} />,
  );
  expect(screen.getByRole("button", { name: "Whiteboard" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  fireEvent.click(screen.getByRole("button", { name: "Presentation" }));
  expect(onSelectSurface).toHaveBeenCalledWith("presentation");
});

test("a manager can reach the end-for-everyone action from the toolbar", () => {
  render(
    <SessionToolbar
      locale="en"
      endAction={<button type="button">End session for everyone</button>}
    />,
  );
  expect(screen.getByRole("button", { name: "Leave session" })).toBeVisible();
  expect(screen.getByRole("button", { name: "End session for everyone" })).toBeVisible();
});
