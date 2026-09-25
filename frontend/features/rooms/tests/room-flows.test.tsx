import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { post, push, refresh } = vi.hoisted(() => ({
  post: vi.fn(),
  push: vi.fn(),
  refresh: vi.fn(),
}));

vi.mock("@/lib/api/browser", () => ({ browserApiClient: { post } }));
vi.mock("next/navigation", () => ({ useRouter: () => ({ push, refresh }) }));

import { CreateRoomForm } from "../create-room-form";

beforeEach(() => {
  post.mockReset();
  push.mockReset();
  refresh.mockReset();
});

test("creates an invite-only room and opens the created room", async () => {
  post.mockResolvedValue({ data: { id: "room-2" } });
  render(<CreateRoomForm locale="en" />);

  fireEvent.change(screen.getByRole("textbox", { name: "Room name" }), {
    target: { value: "  Weekly sync  " },
  });
  fireEvent.click(screen.getByRole("button", { name: "Create room" }));

  await waitFor(() => expect(push).toHaveBeenCalledWith("/room/room-2"));
  expect(post).toHaveBeenCalledWith("/v1/rooms", {
    title: "Weekly sync",
    visibility: "invite_only",
  });
});

test("requires a room name before sending a request", () => {
  render(<CreateRoomForm locale="en" />);
  fireEvent.click(screen.getByRole("button", { name: "Create room" }));

  expect(screen.getByRole("alert")).toHaveTextContent("Enter a room name.");
  expect(post).not.toHaveBeenCalled();
});
