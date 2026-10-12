import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { PartoCompanion } from "../components/parto-companion";
vi.mock("../components/parto", () => ({
  Parto: ({ activity, paused }: { activity: string; paused: boolean }) => (
    <div data-testid="mascot" data-activity={activity} data-paused={String(paused)} />
  ),
}));
beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      disconnect() {}
    },
  );
});
afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});
it("finishes its greeting and cleans up its timer", () => {
  const view = render(<PartoCompanion locale="en" />);
  expect(view.getByTestId("mascot")).toHaveAttribute("data-activity", "speaking");
  act(() => vi.advanceTimersByTime(6500));
  expect(view.getByTestId("mascot")).toHaveAttribute("data-activity", "idle");
  view.unmount();
  expect(vi.getTimerCount()).toBe(0);
});
it("provides a Persian greeting without extra controls", () => {
  const view = render(<PartoCompanion locale="fa" />);
  expect(view.getByText(/سلام، من پرتو/)).toBeVisible();
  expect(view.queryByRole("button")).not.toBeInTheDocument();
});
