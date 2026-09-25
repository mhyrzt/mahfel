import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";

const { patch, refresh } = vi.hoisted(() => ({ patch: vi.fn(), refresh: vi.fn() }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { patch } }));
vi.mock("next/navigation", () => ({ useRouter: () => ({ refresh }) }));

import { SessionSettingsDialog } from "../session-settings-dialog";
import { RoomSessions } from "../room-sessions";

const session = {
  id: "session-1",
  room_id: "room-1",
  title: "Weekly review",
  title_overridden: true,
  status: "scheduled" as const,
  scheduled_start_at: "2026-09-25T09:30:00Z",
  scheduled_duration_minutes: 60,
  created_at: "2026-09-24T09:00:00Z",
  updated_at: "2026-09-25T09:00:00Z",
};

test("manager opens session settings and saves title and schedule", async () => {
  patch.mockResolvedValue({ data: session });
  render(<SessionSettingsDialog session={session} locale="en" />);

  fireEvent.click(screen.getByRole("button", { name: "Session settings: Weekly review" }));
  const dialog = screen.getByRole("dialog", { name: "Session settings" });
  expect(dialog).toBeInTheDocument();
  fireEvent.change(screen.getByRole("textbox", { name: "Session title (optional)" }), {
    target: { value: "Revised review" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(patch).toHaveBeenCalledWith(
      "/v1/sessions/session-1",
      expect.objectContaining({ title: "Revised review", scheduled_duration_minutes: 60 }),
    ),
  );
  expect(refresh).toHaveBeenCalledOnce();
});

test("only users with room session management see a card settings button", () => {
  const fileData = { [session.id]: { files: [], roomFiles: [] } };
  const { rerender } = render(
    <RoomSessions sessions={[session]} locale="en" fileData={fileData} />,
  );
  expect(
    screen.queryByRole("button", { name: "Session settings: Weekly review" }),
  ).not.toBeInTheDocument();
  rerender(<RoomSessions sessions={[session]} locale="en" fileData={fileData} canManageSession />);
  expect(
    screen.getByRole("button", { name: "Session settings: Weekly review" }),
  ).toBeInTheDocument();
});
