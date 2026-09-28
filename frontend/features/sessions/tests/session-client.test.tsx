import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

const { clients } = vi.hoisted(() => ({
  clients: [] as Array<{
    onMessage: (message: unknown) => void;
    onStatus: (status: string) => void;
    connect: ReturnType<typeof vi.fn>;
    send: ReturnType<typeof vi.fn>;
    leave: ReturnType<typeof vi.fn>;
    close: ReturnType<typeof vi.fn>;
  }>,
}));

const { getRoster } = vi.hoisted(() => ({ getRoster: vi.fn() }));
const { push } = vi.hoisted(() => ({ push: vi.fn() }));

vi.mock("next/navigation", () => ({ useRouter: () => ({ push }) }));

vi.mock("@/lib/api/browser", () => ({
  browserApiClient: { get: getRoster },
}));

vi.mock("@/lib/websocket/client", () => ({
  SignalingClient: class {
    readonly connect = vi.fn();
    readonly send = vi.fn();
    readonly leave = vi.fn();
    readonly close = vi.fn();

    constructor(
      _sessionId: string,
      onMessage: (message: unknown) => void,
      onStatus: (status: string) => void,
    ) {
      clients.push({
        onMessage,
        onStatus,
        connect: this.connect,
        send: this.send,
        leave: this.leave,
        close: this.close,
      });
    }
  },
}));

import { SessionClient } from "../components/session-client";

const capabilities = {
  canPublishAudio: true,
  canPublishVideo: true,
  canShareScreen: false,
  canModerate: false,
};

function mountSession() {
  render(
    <SessionClient
      actorId="actor-1"
      capabilities={capabilities}
      locale="en"
      sessionId="session-1"
      roomId="room-1"
      title="Biology 101"
      files={<div />}
    />,
  );
  const client = clients.at(-1);
  if (!client) {
    throw new Error("The signaling client should mount with the session component.");
  }
  return client;
}

function deliver(message: object) {
  act(() => clients.at(-1)?.onMessage(message));
}

beforeEach(() => {
  getRoster.mockResolvedValue({ data: [] });
});

afterEach(() => {
  clients.length = 0;
  getRoster.mockReset();
  push.mockReset();
});

test("leave closes signaling and returns to the room", () => {
  const client = mountSession();
  deliver({ v: 1, type: "joined", session_id: "session-1", participant_id: "part-a111" });
  fireEvent.click(screen.getByRole("button", { name: "Leave session" }));
  expect(client.leave).toHaveBeenCalledOnce();
  expect(client.close).toHaveBeenCalledOnce();
  expect(push).toHaveBeenCalledWith("/room/room-1");
});

test("joined participants can try media controls and see setup failures", async () => {
  mountSession();
  deliver({ v: 1, type: "joined", session_id: "session-1", participant_id: "part-a111" });

  const microphone = screen.getByRole("button", { name: "Unmute microphone" });
  expect(microphone).toBeEnabled();
  fireEvent.click(microphone);
  await waitFor(() =>
    expect(screen.getByRole("alert")).toHaveTextContent("Media connection unavailable"),
  );
});

test("shows the resolved display name in the live roster", async () => {
  getRoster.mockResolvedValue({
    data: [
      {
        id: "part-a111",
        name: "Ada Lovelace",
        role: "member",
        avatar_url: "https://storage.example/avatar.png",
      },
    ],
  });
  mountSession();
  deliver({ v: 1, type: "joined", session_id: "session-1", participant_id: "part-a111" });
  deliver({
    v: 1,
    type: "session_roster",
    session_id: "session-1",
    version: 1,
    participant_ids: ["part-a111"],
  });

  await waitFor(() => expect(screen.getAllByText("Ada Lovelace").length).toBeGreaterThan(0));
});

test("reconciles snapshots and participant events using stable IDs", () => {
  const client = mountSession();

  deliver({
    v: 1,
    type: "session_roster",
    session_id: "session-1",
    version: 4,
    participant_ids: ["part-a111", "part-b222"],
  });
  deliver({ v: 1, type: "joined", session_id: "session-1", participant_id: "part-a111" });

  expect(screen.getAllByText("Participant part")).toHaveLength(4);
  expect(client.connect).toHaveBeenCalledOnce();

  deliver({
    v: 1,
    type: "participant_joined",
    session_id: "session-1",
    participant_id: "part-c333",
  });
  expect(screen.getAllByText("Participant part")).toHaveLength(6);

  deliver({ v: 1, type: "participant_left", session_id: "session-1", participant_id: "part-b222" });
  expect(screen.getAllByText("Participant part")).toHaveLength(4);

  deliver({
    v: 1,
    type: "participant_removed",
    session_id: "session-1",
    participant_id: "part-c333",
  });
  expect(screen.getAllByText("Participant part")).toHaveLength(2);
});

test("shows the lobby without media controls and lets the caller retry joining", () => {
  const client = mountSession();
  deliver({ v: 1, type: "join_waiting", session_id: "session-1", request_id: "request-1" });

  expect(screen.getByText("Waiting for admission")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Mute microphone" })).not.toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "Check admission" }));
  expect(client.send).toHaveBeenCalledWith({ type: "join", session_id: "session-1" });
});

test("turns a blocked-identity event and auth expiry into visible states", () => {
  const client = mountSession();
  deliver({
    v: 1,
    type: "joined",
    session_id: "session-1",
    participant_id: "part-a111",
  });
  deliver({ v: 1, type: "principal_blocked", session_id: "session-1", actor_id: "actor-1" });

  expect(screen.getByText("Access to this session was blocked.")).toBeInTheDocument();
  expect(client.close).toHaveBeenCalledOnce();

  act(() => client.onStatus("unauthorized"));
  expect(screen.getByText("Your sign-in expired. Sign in again to continue.")).toBeInTheDocument();
});

test("opens the sidebar conversation from a new-message toast without leaving the session", async () => {
  getRoster.mockImplementation((url: string) =>
    Promise.resolve({ data: url.endsWith("/messages") ? { messages: [], next_cursor: null } : [] }),
  );
  const client = mountSession();
  act(() => client.onStatus("connected"));
  expect(client.send).toHaveBeenCalledWith({
    type: "subscribe_chat",
    room_id: "room-1",
    after_sequence: 0,
  });
  deliver({ v: 1, type: "chat_subscribed", room_id: "room-1", head_sequence: 3 });
  deliver({
    v: 1,
    type: "chat_event",
    event: {
      room_id: "room-1",
      session_id: "session-1",
      author_id: "actor-2",
      kind: "created",
      sequence: 4,
      content: "Hello from class",
    },
  });
  expect(screen.getByText("Hello from class")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: /New message/ }));
  expect(await screen.findByRole("dialog", { name: "Conversation" })).toBeInTheDocument();
  expect(push).not.toHaveBeenCalled();
});

test("does not notify a session participant about a room-only message", () => {
  getRoster.mockImplementation((url: string) =>
    Promise.resolve({ data: url.endsWith("/messages") ? { messages: [], next_cursor: null } : [] }),
  );
  const client = mountSession();
  act(() => client.onStatus("connected"));
  deliver({ v: 1, type: "chat_subscribed", room_id: "room-1", head_sequence: 3 });
  deliver({
    v: 1,
    type: "chat_event",
    event: {
      room_id: "room-1",
      author_id: "actor-2",
      kind: "created",
      sequence: 4,
      content: "Room-only note",
    },
  });
  expect(screen.queryByText("Room-only note")).not.toBeInTheDocument();
});
