import { afterEach, expect, test, vi } from "vitest";

const { postMock } = vi.hoisted(() => ({ postMock: vi.fn() }));

vi.mock("@/lib/api/browser", () => ({
  browserApiClient: { post: postMock },
}));

import { SignalingClient } from "../client";

class MockWebSocket {
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;
  static readonly instances: MockWebSocket[] = [];

  readyState = 0;
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: (() => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  readonly sent: string[] = [];

  constructor(readonly url: string) {
    MockWebSocket.instances.push(this);
  }

  open(): void {
    this.readyState = MockWebSocket.OPEN;
    this.onopen?.(new Event("open"));
  }

  receive(payload: object): void {
    this.onmessage?.(new MessageEvent("message", { data: JSON.stringify(payload) }));
  }

  send(payload: string): void {
    this.sent.push(payload);
  }

  close(code = 1000): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.({ code } as CloseEvent);
  }
}

afterEach(() => {
  MockWebSocket.instances.length = 0;
  postMock.mockReset();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

test("gets a single-use ticket through the BFF and authenticates before joining", async () => {
  vi.stubGlobal("WebSocket", MockWebSocket);
  postMock.mockResolvedValue({ data: { ticket: "ticket-one", expires_in: 10 } });
  const onMessage = vi.fn();
  const onStatus = vi.fn();
  const client = new SignalingClient("session-123", onMessage, onStatus);

  client.connect();
  await vi.waitFor(() => expect(MockWebSocket.instances).toHaveLength(1));

  const socket = MockWebSocket.instances[0];
  expect(postMock).toHaveBeenCalledWith("/v1/ws/tickets");
  expect(socket.url).toMatch(/^wss?:\/\//);
  socket.open();
  expect(JSON.parse(socket.sent[0])).toEqual({ v: 1, type: "auth", ticket: "ticket-one" });

  socket.receive({ v: 1, type: "welcome", connection_id: "connection-1" });

  expect(JSON.parse(socket.sent[1])).toEqual({
    v: 1,
    type: "join",
    session_id: "session-123",
  });
  expect(onStatus).toHaveBeenCalledWith("connected");
  expect(onMessage).toHaveBeenCalledWith({
    v: 1,
    type: "welcome",
    connection_id: "connection-1",
  });
  client.close();
});

test("requests a fresh ticket and rejoins after a dropped connection", async () => {
  vi.useFakeTimers();
  vi.stubGlobal("WebSocket", MockWebSocket);
  postMock
    .mockResolvedValueOnce({ data: { ticket: "ticket-one", expires_in: 10 } })
    .mockResolvedValueOnce({ data: { ticket: "ticket-two", expires_in: 10 } });
  const client = new SignalingClient("session-123", vi.fn());

  client.connect();
  await vi.waitFor(() => expect(MockWebSocket.instances).toHaveLength(1));
  const first = MockWebSocket.instances[0];
  first.open();
  first.receive({ v: 1, type: "welcome", connection_id: "connection-1" });
  first.close(1006);

  await vi.advanceTimersByTimeAsync(500);
  await vi.waitFor(() => expect(MockWebSocket.instances).toHaveLength(2));
  const second = MockWebSocket.instances[1];
  second.open();

  expect(postMock).toHaveBeenCalledTimes(2);
  expect(JSON.parse(second.sent[0])).toEqual({ v: 1, type: "auth", ticket: "ticket-two" });
  second.receive({ v: 1, type: "welcome", connection_id: "connection-2" });
  expect(JSON.parse(second.sent[1])).toMatchObject({ type: "join", session_id: "session-123" });
  client.close();
});

test("reports an expired browser session without trying a bearer token", async () => {
  vi.stubGlobal("WebSocket", MockWebSocket);
  postMock.mockRejectedValue({ isAxiosError: true, response: { status: 401 } });
  const onStatus = vi.fn();
  const client = new SignalingClient("session-123", vi.fn(), onStatus);

  client.connect();
  await vi.waitFor(() => expect(onStatus).toHaveBeenCalledWith("unauthorized"));

  expect(MockWebSocket.instances).toHaveLength(0);
  client.close();
});

test("correlates media requests and surfaces server errors", async () => {
  vi.stubGlobal("WebSocket", MockWebSocket);
  postMock.mockResolvedValue({ data: { ticket: "ticket-one", expires_in: 10 } });
  const client = new SignalingClient("session-123", vi.fn());
  client.connect();
  await vi.waitFor(() => expect(MockWebSocket.instances).toHaveLength(1));
  const socket = MockWebSocket.instances[0];
  socket.open();
  socket.receive({ v: 1, type: "welcome", connection_id: "connection-1" });

  const created = client.request({ type: "create_transport", room_id: "room-1" });
  const first = JSON.parse(socket.sent.at(-1) ?? "{}") as { id: string };
  socket.receive({
    v: 1,
    id: first.id,
    type: "transport_created",
    room_id: "room-1",
    transport_id: "transport-1",
    ice_parameters: "{}",
    ice_candidates: "[]",
    dtls_parameters: "{}",
    rtp_capabilities: "{}",
  });
  await expect(created).resolves.toMatchObject({
    type: "transport_created",
    transport_id: "transport-1",
  });

  const denied = client.request({ type: "stop_source", room_id: "room-1", source: "microphone" });
  const second = JSON.parse(socket.sent.at(-1) ?? "{}") as { id: string };
  socket.receive({
    v: 1,
    id: second.id,
    type: "error",
    code: "forbidden",
    message: "not permitted",
  });
  await expect(denied).rejects.toThrow("not permitted");
  client.close();
});

test("coalesces connects while the ticket request is pending", async () => {
  vi.stubGlobal("WebSocket", MockWebSocket);
  let resolveTicket!: (value: { data: { ticket: string } }) => void;
  postMock.mockImplementation(
    () =>
      new Promise((resolve) => {
        resolveTicket = resolve;
      }),
  );
  const client = new SignalingClient("session-123", vi.fn());
  client.connect();
  client.connect();
  expect(postMock).toHaveBeenCalledTimes(1);
  resolveTicket({ data: { ticket: "ticket-one" } });
  await vi.waitFor(() => expect(MockWebSocket.instances).toHaveLength(1));
  client.close();
});

test("an old ticket cannot open a socket after close and reconnect", async () => {
  vi.stubGlobal("WebSocket", MockWebSocket);
  let resolveOld!: (value: { data: { ticket: string } }) => void;
  postMock
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveOld = resolve;
        }),
    )
    .mockResolvedValueOnce({ data: { ticket: "fresh-ticket" } });
  const client = new SignalingClient("session-123", vi.fn());
  client.connect();
  client.close();
  client.connect();
  await vi.waitFor(() => expect(MockWebSocket.instances).toHaveLength(1));
  resolveOld({ data: { ticket: "stale-ticket" } });
  await Promise.resolve();
  expect(MockWebSocket.instances).toHaveLength(1);
  const socket = MockWebSocket.instances[0];
  socket.open();
  expect(JSON.parse(socket.sent[0])).toMatchObject({ ticket: "fresh-ticket" });
  client.close();
});
