import { fireEvent, render, screen, within } from "@testing-library/react";
import { expect, test, vi } from "vitest";

const { router } = vi.hoisted(() => ({ router: { refresh: vi.fn() } }));

vi.mock("next/navigation", () => ({ useRouter: () => router }));
vi.mock("next-auth/react", () => ({ signOut: vi.fn() }));
vi.mock("@/features/account/components/device-preferences", () => ({
  DevicePreferences: () => null,
}));

import { AccountQuickSettings } from "../components/account-quick-settings";

test("opens account settings from the avatar in the workspace shell", () => {
  render(
    <AccountQuickSettings
      accountName="Roya Azadi"
      calendar="gregorian"
      email="roya@example.com"
      locale="en"
      theme="system"
    />,
  );

  fireEvent.pointerDown(screen.getByRole("button", { name: "Account" }), {
    button: 0,
    ctrlKey: false,
  });

  const menu = screen.getByRole("menu");
  expect(within(menu).queryByText("Appearance")).not.toBeInTheDocument();
  expect(within(menu).getByText("Roya Azadi")).toBeInTheDocument();
  fireEvent.click(within(menu).getByRole("menuitem", { name: "Open account settings" }));

  const dialog = screen.getByRole("dialog", { name: "Account" });
  expect(within(dialog).getByText("roya@example.com")).toBeInTheDocument();
  expect(within(dialog).getByRole("radiogroup", { name: "Appearance" })).toBeInTheDocument();

  const calendar = within(dialog).getByRole("radiogroup", { name: "Calendar type" });
  fireEvent.click(within(calendar).getByRole("radio", { name: "Persian" }));
  expect(within(calendar).getByRole("radio", { name: "Persian" })).toHaveAttribute(
    "aria-checked",
    "true",
  );
  expect(document.cookie).toContain("mahfel_calendar=persian");
});
