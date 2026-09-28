import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

const { refresh } = vi.hoisted(() => ({ refresh: vi.fn() }));
vi.mock("next/navigation", () => ({ useRouter: () => ({ refresh }) }));

import { SidebarThemeToggle } from "../components/sidebar-theme-toggle";

beforeEach(() => {
  vi.stubGlobal("matchMedia", () => ({
    matches: false,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  }));
});

afterEach(() => {
  document.documentElement.dataset.theme = "system";
  refresh.mockReset();
  vi.unstubAllGlobals();
});

test("the sidebar button switches light and dark themes", () => {
  render(<SidebarThemeToggle initialTheme="light" locale="en" />);

  fireEvent.click(screen.getByRole("button", { name: "Dark" }));
  expect(document.documentElement.dataset.theme).toBe("dark");
  expect(screen.getByRole("button", { name: "Light" })).toBeInTheDocument();
  expect(refresh).toHaveBeenCalled();
});
