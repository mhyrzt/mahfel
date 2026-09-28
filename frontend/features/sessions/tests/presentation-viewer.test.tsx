import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { state, api } = vi.hoisted(() => {
  const state = { index: 0, listeners: new Map<string, Set<() => void>>() };
  const select = (index: number) => {
    state.index = index;
    state.listeners.get("select")?.forEach((listener) => listener());
  };
  const api = {
    selectedScrollSnap: () => state.index,
    canScrollPrev: () => state.index > 0,
    canScrollNext: () => state.index < 3,
    scrollTo: vi.fn((index: number) => select(index)),
    scrollPrev: vi.fn(() => select(Math.max(0, state.index - 1))),
    scrollNext: vi.fn(() => select(Math.min(3, state.index + 1))),
    on: (event: string, listener: () => void) => {
      const listeners = state.listeners.get(event) ?? new Set();
      listeners.add(listener);
      state.listeners.set(event, listeners);
    },
    off: (event: string, listener: () => void) => state.listeners.get(event)?.delete(listener),
  };
  return { state, api };
});

vi.mock("embla-carousel-react", () => ({ default: () => [() => undefined, api] }));

import { PresentationViewer } from "../components/presentation-viewer";

beforeEach(() => {
  state.index = 0;
  state.listeners.clear();
  vi.clearAllMocks();
});

const base = {
  currentPage: 1,
  totalPages: 4,
  canPresent: true,
  live: true,
  locale: "en" as const,
  renderPage: (page: number) => <p>Slide {page}</p>,
  onStop: vi.fn(),
};

test("viewers follow remote pages and cannot navigate with buttons or keyboard", () => {
  const onPageChange = vi.fn();
  const { rerender } = render(
    <PresentationViewer {...base} canPresent={false} onPageChange={onPageChange} />,
  );
  expect(screen.queryByRole("button", { name: "Next" })).not.toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Stop presenting" })).not.toBeInTheDocument();
  fireEvent.keyDown(screen.getByRole("region", { name: "Presentation" }), { key: "ArrowRight" });
  expect(api.scrollNext).not.toHaveBeenCalled();
  rerender(
    <PresentationViewer {...base} currentPage={3} canPresent={false} onPageChange={onPageChange} />,
  );
  expect(api.scrollTo).toHaveBeenCalledWith(2, true);
  expect(onPageChange).not.toHaveBeenCalled();
});

test("presenter navigation updates the shared page", () => {
  const onPageChange = vi.fn();
  render(<PresentationViewer {...base} onPageChange={onPageChange} />);
  fireEvent.click(screen.getByRole("button", { name: "Next" }));
  expect(onPageChange).toHaveBeenCalledWith(2);
});

test("RTL keyboard navigation uses the matching carousel direction", () => {
  state.index = 1;
  render(<PresentationViewer {...base} currentPage={2} locale="fa" onPageChange={vi.fn()} />);
  fireEvent.keyDown(screen.getByRole("region"), { key: "ArrowRight" });
  expect(api.scrollPrev).toHaveBeenCalledOnce();
  expect(api.scrollNext).not.toHaveBeenCalled();
});
