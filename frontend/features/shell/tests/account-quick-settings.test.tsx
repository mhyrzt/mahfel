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

  fireEvent.click(screen.getByRole("button", { name: "Open account settings" }));

  const dialog = screen.getByRole("dialog", { name: "Account" });
  expect(within(dialog).getByText("roya@example.com")).toBeInTheDocument();
});
