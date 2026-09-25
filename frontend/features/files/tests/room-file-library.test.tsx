import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { listFiles, uploadFile } = vi.hoisted(() => ({ listFiles: vi.fn(), uploadFile: vi.fn() }));

vi.mock("../client", () => ({ listFiles, uploadFile, getFileDownloadUrl: vi.fn() }));

import { ContextFileLibrary } from "../components/context-file-library";
import { FileList } from "../components/file-list";

beforeEach(() => {
  listFiles.mockReset();
  uploadFile.mockReset();
});

test("uploads to the room context and refreshes the list", async () => {
  listFiles
    .mockResolvedValueOnce([])
    .mockResolvedValueOnce([
      { id: "file-1", file_name: "notes.pdf", mime: "application/pdf", status: "ready" },
    ]);
  uploadFile.mockResolvedValue({ id: "file-1" });
  render(
    <ContextFileLibrary context={{ kind: "room", id: "room-1" }} locale="en" canDelete={false} />,
  );
  await screen.findByText("No files shared yet.");

  fireEvent.change(screen.getByLabelText("Upload file"), {
    target: { files: [new File(["notes"], "notes.pdf", { type: "application/pdf" })] },
  });

  await waitFor(() => expect(screen.getByText("notes.pdf")).toBeInTheDocument());
  expect(uploadFile).toHaveBeenCalledWith(expect.any(File), { kind: "room", id: "room-1" });
  expect(screen.queryByRole("button", { name: /Delete file/ })).not.toBeInTheDocument();
});

test("a file owner can remove their own asset without room management", async () => {
  listFiles.mockResolvedValue([
    {
      id: "file-1",
      file_name: "notes.pdf",
      owner_id: "actor-1",
      mime: "application/pdf",
      status: "ready",
    },
  ]);
  render(
    <ContextFileLibrary
      context={{ kind: "room", id: "room-1" }}
      locale="en"
      canDelete={false}
      actorId="actor-1"
    />,
  );
  expect(await screen.findByRole("button", { name: "Delete file: notes.pdf" })).toBeInTheDocument();
});

test("uploads an asset in a session context", async () => {
  listFiles.mockResolvedValue([]);
  uploadFile.mockResolvedValue({ id: "file-2" });
  render(
    <ContextFileLibrary
      context={{ kind: "session", id: "session-1" }}
      locale="en"
      canDelete={false}
    />,
  );
  await screen.findByText("No files shared yet.");
  fireEvent.change(screen.getByLabelText("Upload file"), {
    target: { files: [new File(["slide"], "slide.pdf", { type: "application/pdf" })] },
  });
  await waitFor(() =>
    expect(uploadFile).toHaveBeenCalledWith(expect.any(File), { kind: "session", id: "session-1" }),
  );
});

test("file actions follow the owner and manager roles", () => {
  const files = [
    {
      id: "file-1",
      file_name: "notes.pdf",
      owner_id: "actor-1",
      context_id: "room-1",
      context_kind: "room" as const,
      checksum_sha256: "demo",
      created_at: "2026-09-24T09:00:00Z",
      updated_at: "2026-09-24T09:00:00Z",
      mime: "application/pdf",
      size_bytes: 2048,
      status: "ready",
    },
  ];
  const { rerender } = render(
    <FileList files={files} locale="en" canDelete={false} actorId="actor-2" />,
  );
  expect(screen.queryByRole("button", { name: "Delete file: notes.pdf" })).not.toBeInTheDocument();
  rerender(<FileList files={files} locale="en" canDelete={true} actorId="actor-2" />);
  expect(screen.getByRole("button", { name: "Delete file: notes.pdf" })).toBeInTheDocument();
});

test("switches between list and grid and previews a supported file", () => {
  const onPreview = vi.fn();
  const files = [
    {
      id: "file-1",
      file_name: "notes.pdf",
      owner_id: "actor-1",
      context_id: "room-1",
      context_kind: "room" as const,
      checksum_sha256: "demo",
      created_at: "2026-09-24T09:00:00Z",
      updated_at: "2026-09-24T09:00:00Z",
      mime: "application/pdf",
      size_bytes: 2048,
      status: "ready",
    },
  ];
  render(<FileList files={files} locale="en" canDelete={false} onPreview={onPreview} />);
  expect(screen.getByTestId("file-list")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Grid view" }));
  expect(screen.getByTestId("file-grid")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Preview: notes.pdf" }));
  expect(onPreview).toHaveBeenCalledWith(files[0]);
});
