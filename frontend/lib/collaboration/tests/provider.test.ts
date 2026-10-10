import { afterEach, expect, test, vi } from "vitest";
import * as Y from "yjs";

const { postMock } = vi.hoisted(() => ({ postMock: vi.fn() }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { post: postMock } }));
import { MahfelCollaborationProvider } from "../provider";

class MockSocket {
  static readonly OPEN = 1;
  static readonly instances: MockSocket[] = [];
  readyState = 1;
  binaryType = "";
  onopen: (() => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  send = vi.fn();
  close = vi.fn((code = 1000) => {
    this.readyState = 3;
    this.onclose?.({ code } as CloseEvent);
  });
  constructor(readonly url: string) {
    MockSocket.instances.push(this);
  }
}

let provider: MahfelCollaborationProvider | undefined;
let doc: Y.Doc | undefined;
afterEach(() => {
  provider?.destroy();
  doc?.destroy();
  provider = undefined;
  doc = undefined;
  MockSocket.instances.length = 0;
  vi.unstubAllGlobals();
  vi.useRealTimers();
  postMock.mockReset();
});

async function connect() {
  vi.stubGlobal("WebSocket", MockSocket);
  postMock.mockResolvedValue({ data: { ticket: "fixture-ticket" } });
  doc = new Y.Doc();
  provider = new MahfelCollaborationProvider("session-1", "notes", doc, vi.fn(), vi.fn());
  const status = vi.fn();
  provider.on("status", status);
  provider.connect();
  await Promise.resolve();
  return { socket: MockSocket.instances[0], status };
}

test("malformed JSON closes the connection without throwing from the handler", async () => {
  const { socket } = await connect();
  expect(() => socket.onmessage?.(new MessageEvent("message", { data: "{" }))).not.toThrow();
  expect(socket.close).toHaveBeenCalledWith(1002, "invalid collaboration frame");
});

test("authentication has a deadline and does not reconnect after refusal", async () => {
  vi.useFakeTimers();
  const { socket, status } = await connect();
  await vi.advanceTimersByTimeAsync(10_000);
  expect(socket.close).toHaveBeenCalledWith(4401, "authentication timeout");
  expect(status).toHaveBeenCalledWith({ status: "unauthorized" });
  await vi.advanceTimersByTimeAsync(30_000);
  expect(MockSocket.instances).toHaveLength(1);
});

test("a ready frame cancels the authentication deadline", async () => {
  vi.useFakeTimers();
  const { socket, status } = await connect();
  socket.onmessage?.(
    new MessageEvent("message", {
      data: JSON.stringify({ type: "collaboration_ready", can_edit: true, live: true }),
    }),
  );
  expect(status).toHaveBeenCalledWith({ status: "connected" });
  await vi.advanceTimersByTimeAsync(10_000);
  expect(socket.close).not.toHaveBeenCalled();
});
