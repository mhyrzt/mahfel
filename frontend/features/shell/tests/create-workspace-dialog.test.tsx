import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, post, push, refresh } = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
  push: vi.fn(),
  refresh: vi.fn(),
}));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get, post } }));
vi.mock("next/navigation", () => ({ useRouter: () => ({ push, refresh }) }));

import { CreateWorkspaceDialog } from "../components/create-workspace-dialog";

beforeEach(() => {
  get.mockReset();
  post.mockReset();
  push.mockReset();
  refresh.mockReset();
});

test("creates a room inside the dialog and opens its detail route", async () => {
  const onOpenChange = vi.fn();
  post.mockResolvedValue({ data: { id: "room-2" } });
  render(<CreateWorkspaceDialog kind="room" locale="en" open onOpenChange={onOpenChange} />);
  fireEvent.change(screen.getByRole("textbox", { name: "Room name" }), {
    target: { value: "Weekly sync" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Create room" }));
  await waitFor(() => expect(push).toHaveBeenCalledWith("/room/room-2"));
  expect(onOpenChange).toHaveBeenCalledWith(false);
});

test("loads available rooms before showing the session form", async () => {
  get.mockImplementation((path: string) =>
    path === "/v1/rooms"
      ? Promise.resolve({ data: { data: [{ id: "room-1", title: "Design review" }] } })
      : Promise.resolve({ data: { can_manage_session: true } }),
  );
  render(<CreateWorkspaceDialog kind="session" locale="en" open onOpenChange={() => undefined} />);
  expect(screen.getByTestId("create-session-loading")).toBeInTheDocument();
  expect(await screen.findByTestId("create-session-form")).toBeInTheDocument();
  await waitFor(() => expect(screen.getByRole("button", { name: "New session" })).toBeEnabled());
});
