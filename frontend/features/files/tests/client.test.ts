import { afterEach, expect, test, vi } from "vitest";

import { browserApiClient } from "@/lib/api/browser";
import { uploadFile } from "../client";

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

test("uploads bytes directly to S3 before completing metadata", async () => {
  const file = new File(["deck"], "deck.pdf", { type: "application/pdf" });
  vi.spyOn(file, "arrayBuffer").mockResolvedValue(new Uint8Array([1, 2, 3]).buffer);
  vi.stubGlobal("crypto", {
    subtle: { digest: vi.fn().mockResolvedValue(new Uint8Array(32).buffer) },
  });
  const signed = {
    url: "https://s3.example.test/bucket/uploads/id",
    headers: [
      { name: "content-type", value: "application/pdf" },
      { name: "content-length", value: "4" },
    ],
    file: { id: "file-id" },
  };
  const post = vi.spyOn(browserApiClient, "post");
  post.mockResolvedValueOnce({ data: signed });
  post.mockResolvedValueOnce({ data: { id: "file-id", status: "ready" } });
  const transfer = vi.fn().mockResolvedValue({ ok: true, status: 200 });
  vi.stubGlobal("fetch", transfer);

  const result = await uploadFile(file, { kind: "room", id: "room-id" });

  expect(post).toHaveBeenNthCalledWith(
    1,
    "/v1/files/uploads",
    expect.objectContaining({
      checksum_sha256: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
      size_bytes: 4,
    }),
  );
  expect(transfer).toHaveBeenCalledWith(
    signed.url,
    expect.objectContaining({
      method: "PUT",
      body: file,
      credentials: "omit",
    }),
  );
  const headers = transfer.mock.calls[0]?.[1]?.headers as Headers;
  expect(headers.get("content-type")).toBe("application/pdf");
  expect(headers.has("content-length")).toBe(false);
  expect(post).toHaveBeenNthCalledWith(2, "/v1/files/file-id/complete");
  expect(result.status).toBe("ready");
});

test("does not complete metadata after S3 rejects an upload", async () => {
  const file = new File(["bad"], "deck.pdf", { type: "application/pdf" });
  vi.spyOn(file, "arrayBuffer").mockResolvedValue(new Uint8Array([1]).buffer);
  vi.stubGlobal("crypto", {
    subtle: { digest: vi.fn().mockResolvedValue(new Uint8Array(32).buffer) },
  });
  const post = vi.spyOn(browserApiClient, "post").mockResolvedValueOnce({
    data: { url: "https://s3.example.test/upload", headers: [], file: { id: "file-id" } },
  });
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: false, status: 400 }));

  await expect(uploadFile(file, { kind: "room", id: "room-id" })).rejects.toThrow(
    "File transfer failed (400)",
  );
  expect(post).toHaveBeenCalledTimes(1);
});
