import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { expect, test } from "vitest";
import { InteractiveToolbarPreview } from "../stories/session-toolbar.stories";

test("interactive toolbar preview toggles hands and renders the chosen reaction", async () => {
  render(<InteractiveToolbarPreview locale="en" />);
  await act(async () => {});
  fireEvent.click(screen.getByRole("button", { name: "Raise hand" }));
  expect(screen.getByRole("button", { name: "Lower hand" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  fireEvent.keyDown(screen.getByRole("button", { name: "Reactions" }), { key: "Enter" });
  fireEvent.click(await screen.findByRole("menuitem", { name: "Like" }));
  await waitFor(() => expect(screen.getByText("Sara")).toBeInTheDocument());
  expect(screen.getByText("👍")).toBeInTheDocument();
});

test("unavailable preview explicitly disables both interaction controls", () => {
  render(<InteractiveToolbarPreview disabled locale="en" />);
  expect(screen.getByRole("button", { name: "Raise hand" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Reactions" })).toBeDisabled();
});

test("media unavailability does not prevent hand or reaction interactions", () => {
  render(<InteractiveToolbarPreview mediaUnavailable locale="en" />);
  expect(screen.getByRole("button", { name: "Unmute microphone" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Raise hand" })).toBeEnabled();
  expect(screen.getByRole("button", { name: "Reactions" })).toBeEnabled();
});
