import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, remove } = vi.hoisted(() => ({ get: vi.fn(), remove: vi.fn() }));

vi.mock("@/lib/api/browser", () => ({
  browserApiClient: { get, post: vi.fn(), patch: vi.fn(), put: vi.fn(), delete: remove },
}));
vi.mock("@/features/files/client", () => ({ uploadFile: vi.fn() }));
vi.mock("@/lib/websocket/client", () => ({
  SignalingClient: class {
    connect() {}
    close() {}
  },
}));

import { RoomChat } from "../components/room-chat";

beforeEach(() => {
  get.mockReset();
  remove.mockReset();
});

test("deletes an author's message from the overflow menu", async () => {
  const original = {
    id: "message-2",
    channel_id: "channel-1",
    author_id: "actor-1",
    content: "Remove me",
    created_at: "2026-09-25T00:00:00Z",
  };
  get.mockResolvedValue({ data: { messages: [original], next_cursor: null } });
  remove.mockResolvedValue({
    data: { ...original, content: null, deleted_at: "2026-09-25T00:01:00Z" },
  });
  render(<RoomChat roomId="room-1" locale="en" actorId="actor-1" />);

  await screen.findByText("Remove me");
  fireEvent.pointerDown(screen.getByRole("button", { name: "Message actions" }), {
    button: 0,
    ctrlKey: false,
  });
  fireEvent.click(await screen.findByRole("menuitem", { name: "Delete" }));

  await waitFor(() => expect(remove).toHaveBeenCalledWith("/v1/rooms/room-1/messages/message-2"));
  expect(await screen.findByText("Message removed")).toBeInTheDocument();
});
