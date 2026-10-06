import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";
import { SessionBreadcrumb } from "../components/session-breadcrumb";

test("includes a localized view as the current breadcrumb level", () => {
  const props = { roomId: "room", roomName: "Research", title: "Weekly session" };
  const { rerender } = render(
    <SessionBreadcrumb {...props} locale="en" activeSurface="whiteboard" />,
  );
  expect(screen.getByText("Whiteboard")).toHaveAttribute("aria-current", "page");
  expect(screen.getByText("Weekly session")).not.toHaveAttribute("aria-current");
  rerender(<SessionBreadcrumb {...props} locale="fa" activeSurface="activities" />);
  expect(screen.getByText("نظرسنجی و آزمون")).toHaveAttribute("aria-current", "page");
  expect(screen.getByRole("navigation")).toHaveAttribute("dir", "rtl");
});
