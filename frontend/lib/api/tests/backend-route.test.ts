import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("@/lib/auth/server", () => ({
  getServerAccessToken: vi.fn().mockResolvedValue("fixture-token"),
}));
vi.mock("@/lib/api/config", () => ({ serverApiBaseUrl: "https://api.example" }));
import { GET, POST } from "@/app/api/backend/[...path]/route";

const endpoint = "https://web.example/api/backend/v1/rooms";
const context = (path = ["v1", "rooms"]) => ({ params: Promise.resolve({ path }) });
const fetchMock = vi.fn();
beforeEach(() => {
  fetchMock.mockReset().mockResolvedValue(new Response("ok"));
  vi.stubGlobal("fetch", fetchMock);
});
afterEach(() => vi.unstubAllGlobals());

test.each([undefined, "1"])(
  "rejects oversized actual bodies regardless of declared length %s",
  async (length) => {
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(new Uint8Array(1024 * 1024));
        controller.enqueue(new Uint8Array(1024 * 1024 + 1));
        controller.close();
      },
    });
    const init: RequestInit & { duplex: "half" } = {
      method: "POST",
      body: stream,
      duplex: "half",
      headers: length ? { "content-length": length } : {},
    };
    const response = await POST(new Request(endpoint, init), context());
    expect(response.status).toBe(413);
    expect(fetchMock).not.toHaveBeenCalled();
  },
);
test("forwards an allowed bounded body unchanged", async () => {
  await POST(new Request(endpoint, { method: "POST", body: "payload" }), context());
  const init = fetchMock.mock.calls[0][1] as RequestInit;
  expect(await new Response(init.body).text()).toBe("payload");
});
test("drops forwarding, hop-by-hop, and connection-nominated headers in both directions", async () => {
  fetchMock.mockResolvedValue(
    new Response("ok", {
      headers: {
        connection: "x-private",
        "x-private": "hidden",
        "keep-alive": "timeout=5",
        "set-cookie": "upstream=1",
        "x-request-id": "id",
      },
    }),
  );
  const request = new Request(endpoint, {
    headers: {
      connection: "x-private",
      "x-private": "hidden",
      forwarded: "for=spoofed",
      "x-forwarded-for": "spoofed",
      "x-real-ip": "spoofed",
      "proxy-authorization": "private",
      te: "trailers",
      "x-request-id": "id",
    },
  });
  const response = await GET(request, context());
  const headers = new Headers(fetchMock.mock.calls[0][1].headers);
  for (const name of [
    "connection",
    "x-private",
    "forwarded",
    "x-forwarded-for",
    "x-real-ip",
    "proxy-authorization",
    "te",
  ]) {
    expect(headers.has(name)).toBe(false);
  }
  expect(headers.get("authorization")).toBe("Bearer fixture-token");
  expect(headers.get("x-request-id")).toBe("id");
  for (const name of ["connection", "x-private", "keep-alive", "set-cookie"]) {
    expect(response.headers.has(name)).toBe(false);
  }
  expect(response.headers.get("x-request-id")).toBe("id");
});
test.each([
  { path: ["v1", "rooms", "..", "..", "admin"] },
  { path: ["v1", "rooms", "%2e%2e", "admin"] },
  { path: ["v1", "rooms", "%252e%252e", "admin"] },
  { path: ["v1", "rooms", "..\\admin"] },
  { path: ["v1", "rooms", "../admin"] },
])("rejects unsafe route segments $path", async ({ path }) => {
  const response = await GET(new Request(endpoint), context(path));
  expect(response.status).toBe(404);
  expect(fetchMock).not.toHaveBeenCalled();
});
