import { fireEvent, render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";

import { PageSkeleton } from "../components/page-skeleton";
import { RouteError } from "../components/route-error";

test("route error offers a retry without showing technical error details", () => {
  const reset = vi.fn();
  render(<RouteError scope="rooms" locale="en" reset={reset} />);
  expect(screen.getByRole("heading", { name: "Rooms could not be loaded" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Try again" }));
  expect(reset).toHaveBeenCalledOnce();
  expect(screen.getByRole("link", { name: "Go to Mahfel" })).toHaveAttribute("href", "/");
});

test("route error supports Persian", () => {
  render(<RouteError scope="sessions" locale="fa" reset={() => undefined} />);
  expect(screen.getByRole("heading", { name: "بارگذاری دیدارها ممکن نشد" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "تلاش دوباره" })).toBeInTheDocument();
});

test("route error reads the active document language", () => {
  const previous = document.documentElement.lang;
  document.documentElement.lang = "fa";
  try {
    render(<RouteError scope="workspace" reset={() => undefined} />);
    expect(
      screen.getByRole("heading", { name: "بارگذاری فضای کاری ممکن نشد" }),
    ).toBeInTheDocument();
  } finally {
    document.documentElement.lang = previous;
  }
});

test("page skeleton matches the route shape and keeps reduced motion support", () => {
  const { rerender } = render(<PageSkeleton shape="detail" />);
  expect(screen.getByTestId("detail-loading")).toHaveAttribute("aria-busy", "true");
  expect(screen.getByTestId("detail-loading").querySelector(".animate-pulse")).toHaveClass(
    "motion-reduce:animate-none",
  );
  rerender(<PageSkeleton shape="session" />);
  expect(screen.getByTestId("session-loading")).toBeInTheDocument();
});
