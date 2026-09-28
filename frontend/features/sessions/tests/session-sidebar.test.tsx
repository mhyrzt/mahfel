import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { expect, test, vi } from "vitest";

vi.mock("next/navigation", () => ({
  usePathname: () => "/session/session-demo",
  useRouter: () => ({ refresh: vi.fn() }),
}));
vi.mock("next-auth/react", () => ({ signOut: vi.fn() }));
vi.mock("@/features/account/components/device-preferences", () => ({
  DevicePreferences: () => null,
}));

import { AppShell } from "@/features/shell/components/app-shell";
import { SessionSidebar } from "../components/session-sidebar";

test("a session rail button opens its panel and collapses it on a second click", async () => {
  render(
    <>
      <div id="session-sidebar-actions" />
      <SessionSidebar
        locale="fa"
        showAdmission
        participants={<div>Roster content</div>}
        admission={<div>Admission content</div>}
        access={<div>Access content</div>}
        files={<div>File content</div>}
        invites={<div>Invite links content</div>}
      >
        <div>Session stage</div>
      </SessionSidebar>
    </>,
  );

  const rail = document.getElementById("session-sidebar-actions");
  expect(rail).not.toBeNull();
  const files = await waitFor(() =>
    within(rail as HTMLElement).getByRole("button", { name: "فایل‌ها" }),
  );
  expect(files).toHaveAttribute("aria-pressed", "false");
  fireEvent.click(files);
  expect(files).toHaveAttribute("aria-pressed", "true");
  expect(screen.getByText("File content")).toBeVisible();
  fireEvent.click(files);
  expect(files).toHaveAttribute("aria-pressed", "false");
  const invites = within(rail as HTMLElement).getByRole("button", { name: "پیوندهای دعوت" });
  fireEvent.click(invites);
  expect(invites).toHaveAttribute("aria-pressed", "true");
  expect(screen.getByText("Invite links content")).toBeVisible();
});

test("room, session, and account controls remain in the shell rail beside a session panel", async () => {
  render(
    <AppShell accountName="Misato Katsuragi" locale="fa">
      <SessionSidebar
        locale="fa"
        showAdmission={false}
        participants={<div>Roster content</div>}
        admission={null}
        access={null}
      >
        <div>Session stage</div>
      </SessionSidebar>
    </AppShell>,
  );

  const rail = screen.getByRole("complementary");
  const rooms = within(rail).getByTestId("browse-rooms");
  const sessions = within(rail).getByTestId("browse-sessions");
  const account = within(rail).getByRole("button", { name: "حساب" });
  const participants = await waitFor(() =>
    within(rail).getByRole("button", { name: "شرکت‌کنندگان" }),
  );

  expect(rooms).toBeVisible();
  expect(sessions).toBeVisible();
  expect(account).toBeVisible();
  fireEvent.click(participants);
  expect(rooms).toBeVisible();
  expect(sessions).toBeVisible();
  expect(account).toBeVisible();
});
