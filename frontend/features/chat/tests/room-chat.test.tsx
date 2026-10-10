import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, post, patch, remove, put, socket, uploadFile } = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
  patch: vi.fn(),
  remove: vi.fn(),
  put: vi.fn(),
  uploadFile: vi.fn(),
  socket: { onMessage: null as null | ((message: unknown) => void) },
}));

vi.mock("@/lib/api/browser", () => ({
  browserApiClient: { get, post, patch, put, delete: remove },
}));
vi.mock("@/features/files/client", () => ({ uploadFile }));
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
  uploadFile.mockReset();
  put.mockResolvedValue({});
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
  fireEvent.pointerDown(screen.getByRole("button", { name: "Message actions" }), {
    button: 0,
    ctrlKey: false,
  });
  fireEvent.click(await screen.findByRole("menuitem", { name: "Edit" }));
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

test("scopes a session conversation and ignores other session events", async () => {
  get.mockResolvedValue({ data: { messages: [], next_cursor: null } });
  render(<RoomChat roomId="room-1" sessionId="session-1" locale="en" actorId="actor-1" />);
  await screen.findByText("No messages yet. Start the conversation.");
  expect(get).toHaveBeenCalledWith("/v1/rooms/room-1/messages", {
    params: { limit: 50, session_id: "session-1" },
  });
  const initialCalls = get.mock.calls.length;
  act(() =>
    socket.onMessage?.({
      type: "chat_event",
      event: { room_id: "room-1", session_id: "session-2", sequence: 3 },
    }),
  );
  expect(get).toHaveBeenCalledTimes(initialCalls);
  act(() =>
    socket.onMessage?.({
      type: "chat_event",
      event: { room_id: "room-1", session_id: "session-1", sequence: 4 },
    }),
  );
  await waitFor(() => expect(get.mock.calls.length).toBeGreaterThan(initialCalls));
});

test("uploads composer attachments against the sent message", async () => {
  get.mockResolvedValue({ data: { messages: [], next_cursor: null } });
  post.mockResolvedValue({
    data: {
      id: "message-9",
      channel_id: "channel-1",
      author_id: "actor-1",
      content: "Report",
      created_at: "2026-09-25T00:00:00Z",
    },
  });
  uploadFile.mockResolvedValue({
    id: "file-9",
    file_name: "clip.mp4",
    mime: "video/mp4",
    size_bytes: 4,
  });
  render(<RoomChat roomId="room-1" locale="en" actorId="actor-1" />);

  await screen.findByText("No messages yet. Start the conversation.");
  fireEvent.change(screen.getByLabelText("Write a message"), {
    target: { value: "Report" },
  });
  fireEvent.change(screen.getByLabelText("Attach files", { selector: "input" }), {
    target: { files: [new File(["clip"], "clip.mp4", { type: "video/mp4" })] },
  });
  fireEvent.click(screen.getByRole("button", { name: "Send" }));

  await waitFor(() =>
    expect(uploadFile).toHaveBeenCalledWith(expect.any(File), {
      kind: "chat_message",
      id: "message-9",
    }),
  );
  expect(post).toHaveBeenCalledWith("/v1/rooms/room-1/messages", { content: "Report" });
});

function chatEvent(sequence: number, content: string | null, deletedAt?: string) {
  return {
    type: "chat_event",
    event: {
      room_id: "room-1",
      sequence,
      kind: deletedAt ? "deleted" : "edited",
      message_id: "live-message",
      author_id: "actor-1",
      created_at: "2026-10-07T10:00:00Z",
      content,
      deleted_at: deletedAt,
    },
  };
}

test("patches ordered realtime messages without reloading history and ignores duplicates", async () => {
  get.mockResolvedValue({ data: { messages: [], next_cursor: null } });
  render(<RoomChat roomId="room-1" locale="en" />);
  await screen.findByText("No messages yet. Start the conversation.");
  const historyCalls = get.mock.calls.length;
  act(() => socket.onMessage?.(chatEvent(1, "Realtime creation")));
  expect(await screen.findByText("Realtime creation")).toBeInTheDocument();
  act(() => socket.onMessage?.(chatEvent(2, "Realtime edit")));
  expect(await screen.findByText("Realtime edit")).toBeInTheDocument();
  act(() => socket.onMessage?.(chatEvent(1, "Stale duplicate")));
  expect(screen.queryByText("Stale duplicate")).not.toBeInTheDocument();
  act(() => socket.onMessage?.(chatEvent(3, null, "2026-10-07T10:01:00Z")));
  expect(await screen.findByText("Message removed")).toBeInTheDocument();
  expect(get).toHaveBeenCalledTimes(historyCalls);
});

test("fetches only the changed message attachments without reloading history", async () => {
  get.mockResolvedValue({ data: { messages: [], next_cursor: null } });
  render(<RoomChat roomId="room-1" locale="en" />);
  await screen.findByText("No messages yet. Start the conversation.");
  const historyCalls = get.mock.calls.filter(([path]) => path.endsWith("/messages")).length;
  get.mockResolvedValue({
    data: { files: [{ id: "file-1", file_name: "notes.txt", mime: "text/plain", size_bytes: 10 }] },
  });
  const event = chatEvent(1, "Attached document");
  act(() =>
    socket.onMessage?.({ ...event, event: { ...event.event, attachment_ids: ["file-1"] } }),
  );
  expect(await screen.findByText("notes.txt")).toBeInTheDocument();
  expect(get).toHaveBeenCalledWith("/v1/files", {
    params: { context_kind: "chat_message", context_id: "live-message", limit: 4 },
  });
  expect(get.mock.calls.filter(([path]) => path.endsWith("/messages"))).toHaveLength(historyCalls);
});
