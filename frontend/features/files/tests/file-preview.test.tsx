import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";

const { listFiles, getFileDownloadUrl } = vi.hoisted(() => ({
  listFiles: vi.fn(),
  getFileDownloadUrl: vi.fn(),
}));
vi.mock("../client", () => ({ listFiles, getFileDownloadUrl, uploadFile: vi.fn() }));

import { ContextFileLibrary } from "../components/context-file-library";

test("previews a signed PDF through a revocable local URL", async () => {
  listFiles.mockResolvedValue([
    {
      id: "file-1",
      file_name: "notes.pdf",
      owner_id: "actor-1",
      context_id: "room-1",
      context_kind: "room",
      checksum_sha256: "demo",
      created_at: "2026-09-24T09:00:00Z",
      updated_at: "2026-09-24T09:00:00Z",
      mime: "application/pdf",
      size_bytes: 2048,
      status: "ready",
    },
  ]);
  getFileDownloadUrl.mockResolvedValue("https://files.example.test/notes.pdf");
  const transfer = vi.fn().mockResolvedValue({
    ok: true,
    blob: () => Promise.resolve(new Blob(["pdf"], { type: "application/pdf" })),
  });
  vi.stubGlobal("fetch", transfer);
  const createDescriptor = Object.getOwnPropertyDescriptor(URL, "createObjectURL");
  const revokeDescriptor = Object.getOwnPropertyDescriptor(URL, "revokeObjectURL");
  const revoke = vi.fn();
  Object.defineProperty(URL, "createObjectURL", {
    configurable: true,
    value: () => "blob:preview",
  });
  Object.defineProperty(URL, "revokeObjectURL", { configurable: true, value: revoke });
  try {
    render(
      <ContextFileLibrary context={{ kind: "room", id: "room-1" }} locale="en" canDelete={false} />,
    );
    fireEvent.click(await screen.findByRole("button", { name: "Preview: notes.pdf" }));
    await waitFor(() =>
      expect(screen.getByRole("dialog").querySelector("iframe")).toHaveAttribute(
        "src",
        "blob:preview",
      ),
    );
    expect(getFileDownloadUrl).toHaveBeenCalledWith("file-1");
    expect(transfer).toHaveBeenCalledWith(new URL("https://files.example.test/notes.pdf"), {
      credentials: "omit",
      redirect: "error",
    });
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    await waitFor(() => expect(revoke).toHaveBeenCalledWith("blob:preview"));
  } finally {
    cleanup();
    if (createDescriptor) {
      Object.defineProperty(URL, "createObjectURL", createDescriptor);
    } else {
      Reflect.deleteProperty(URL, "createObjectURL");
    }
    if (revokeDescriptor) {
      Object.defineProperty(URL, "revokeObjectURL", revokeDescriptor);
    } else {
      Reflect.deleteProperty(URL, "revokeObjectURL");
    }
    vi.unstubAllGlobals();
  }
});
