import { beforeEach, expect, test, vi } from "vitest";

const { uploadFile, post, get } = vi.hoisted(() => ({
  uploadFile: vi.fn(),
  post: vi.fn(),
  get: vi.fn(),
}));
vi.mock("@/features/files/client", () => ({ uploadFile }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { post, get } }));

import { fetchCollaborationImage, registerCollaborationImage } from "./images";

beforeEach(() => {
  uploadFile.mockReset();
  post.mockReset();
  get.mockReset();
});

test("registers a verified session image with the selected document", async () => {
  uploadFile.mockResolvedValue({ id: "file-1" });
  post.mockResolvedValue({});
  const file = new File(["png"], "diagram.png", { type: "image/png" });
  expect(await registerCollaborationImage(file, "session-1", "notes")).toBe("file-1");
  expect(uploadFile).toHaveBeenCalledWith(file, { kind: "session", id: "session-1" });
  expect(post).toHaveBeenCalledWith("/v1/sessions/session-1/collaboration/notes/images", {
    file_id: "file-1",
  });
});

test("rejects unsupported images before uploading", async () => {
  await expect(
    registerCollaborationImage(
      new File(["svg"], "x.svg", { type: "image/svg+xml" }),
      "session-1",
      "whiteboard",
    ),
  ).rejects.toThrow("Only PNG and JPEG");
  expect(uploadFile).not.toHaveBeenCalled();
});

test("downloads registered image bytes without browser credentials", async () => {
  get.mockResolvedValue({
    data: { url: "https://files.example.test/image.png", file: { mime: "image/png" } },
  });
  const transfer = vi.fn().mockResolvedValue({
    ok: true,
    blob: () => Promise.resolve(new Blob(["image"], { type: "image/png" })),
  });
  vi.stubGlobal("fetch", transfer);
  expect((await fetchCollaborationImage("file-1")).type).toBe("image/png");
  expect(transfer).toHaveBeenCalledWith(new URL("https://files.example.test/image.png"), {
    credentials: "omit",
    redirect: "error",
  });
  vi.unstubAllGlobals();
});
