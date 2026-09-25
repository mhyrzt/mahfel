import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, post, patch, remove, put, socket } = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
  patch: vi.fn(),
  remove: vi.fn(),
  put: vi.fn(),
  socket: { onMessage: null as null | ((message: unknown) => void) },
}));

vi.mock("@/lib/api/browser", () => ({
  browserApiClient: { get, post, patch, put, delete: remove },
}));
vi.mock("@/lib/websocket/client", () => ({
  SignalingClient: class {
    constructor(_onSession: unknown, onMessage: (message: unknown) => void) {
      socket.onMessage = onMessage;
    }
    connect() {}
    close() {}
  },
}));

import { RoomChat } from "../components/room-chat";

beforeEach(() => {
  get.mockReset();
  post.mockReset();
  patch.mockReset();
  remove.mockReset();
  put.mockReset();
  socket.onMessage = null;
});

test("advances the read position when a visible room receives a chat event", async () => {
  get.mockResolvedValue({ data: { messages: [], next_cursor: null } });
  put.mockResolvedValue({});
  render(<RoomChat roomId="room-1" locale="en" />);
  await screen.findByText("No messages yet. Start the conversation.");
  act(() => socket.onMessage?.({ type: "chat_event", event: { room_id: "room-1", sequence: 12 } }));
  await waitFor(() =>
    expect(put).toHaveBeenCalledWith("/v1/rooms/room-1/read-marker", { sequence: 12 }),
  );
});

test("lets an author edit their own message", async () => {
  const original = {
    id: "message-1",
    channel_id: "channel-1",
    author_id: "actor-1",
    content: "Before",
    created_at: "2026-09-25T00:00:00Z",
  };
  get.mockResolvedValue({ data: { messages: [original], next_cursor: null } });
  patch.mockResolvedValue({ data: { ...original, content: "After" } });
  render(<RoomChat roomId="room-1" locale="en" actorId="actor-1" />);

  await screen.findByText("Before");
  fireEvent.click(screen.getByRole("button", { name: "Edit" }));
  fireEvent.change(screen.getByRole("textbox", { name: "Edit" }), {
    target: { value: "After" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));

  await waitFor(() => expect(patch).toHaveBeenCalled());
  expect(await screen.findByText("After")).toBeInTheDocument();
  expect(patch).toHaveBeenCalledWith("/v1/rooms/room-1/messages/message-1", { content: "After" });
});

test("loads room history and sends a new message", async () => {
  get.mockResolvedValue({ data: { messages: [], next_cursor: null } });
  post.mockResolvedValue({
    data: {
      id: "message-1",
      channel_id: "channel-1",
      author_id: "actor-1",
      content: "Hello",
      created_at: "2026-09-25T00:00:00Z",
    },
  });
  render(<RoomChat roomId="room-1" locale="en" />);

  await screen.findByText("No messages yet. Start the conversation.");
  fireEvent.change(screen.getByRole("textbox", { name: "Write a message" }), {
    target: { value: " Hello " },
  });
  fireEvent.click(screen.getByRole("button", { name: "Send" }));

  await waitFor(() => expect(screen.getByText("Hello")).toBeInTheDocument());
  expect(post).toHaveBeenCalledWith("/v1/rooms/room-1/messages", { content: "Hello" });
});

test("shows a deliberate history error", async () => {
  get.mockRejectedValue(new Error("offline"));
  render(<RoomChat roomId="room-1" locale="en" />);
  expect(await screen.findByRole("alert")).toHaveTextContent("Messages could not be loaded.");
});
