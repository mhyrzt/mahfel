import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";

import { SessionToolbar } from "../components/session-toolbar";

test("media controls stay disabled until a real RTC handler is connected", () => {
  render(<SessionToolbar locale="en" />);
  expect(screen.getByRole("button", { name: "Unmute microphone" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Turn camera on" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Share screen" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Leave session" })).toBeEnabled();
});
