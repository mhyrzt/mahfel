import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { listFiles, uploadFile, post } = vi.hoisted(() => ({
  listFiles: vi.fn(),
  uploadFile: vi.fn(),
  post: vi.fn(),
}));
vi.mock("@/features/files/client", () => ({ listFiles, uploadFile }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { post } }));

import { SessionFilePicker } from "../session-file-picker";

const roomFile = {
  id: "file-1",
  file_name: "Reading list.pdf",
  context_kind: "room" as const,
  context_id: "room-1",
  owner_id: "actor-1",
  checksum_sha256: "hash",
  created_at: "2026-09-24T09:00:00Z",
  updated_at: "2026-09-24T09:00:00Z",
  mime: "application/pdf",
  size_bytes: 1024,
  status: "ready",
};

beforeEach(() => {
  listFiles.mockReset();
  uploadFile.mockReset();
  post.mockReset();
});

test("selects an existing room file as a persistent session attachment", async () => {
  listFiles.mockResolvedValue([roomFile]);
  post.mockResolvedValue({ data: roomFile });
  const onAdded = vi.fn();
  render(
    <SessionFilePicker
      sessionId="session-1"
      roomId="room-1"
      locale="en"
      attachedIds={[]}
      onAdded={onAdded}
    />,
  );

  fireEvent.click(screen.getByRole("button", { name: /Add a session file/ }));
  fireEvent.click(screen.getByRole("button", { name: "Select from room bank" }));
  expect(await screen.findByRole("button", { name: "Reading list.pdf" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Reading list.pdf" }));
  await waitFor(() =>
    expect(post).toHaveBeenCalledWith("/v1/files/session-links", {
      file_id: "file-1",
      session_id: "session-1",
    }),
  );
  expect(onAdded).toHaveBeenCalledWith(roomFile);
});

test("accepts a dropped file for direct session upload", async () => {
  uploadFile.mockResolvedValue(roomFile);
  const onAdded = vi.fn();
  render(
    <SessionFilePicker
      sessionId="session-1"
      roomId="room-1"
      locale="en"
      attachedIds={[]}
      onAdded={onAdded}
    />,
  );
  const file = new File(["content"], "notes.txt", { type: "text/plain" });

  fireEvent.drop(screen.getByRole("button", { name: /Add a session file/ }), {
    dataTransfer: { files: [file] },
  });
  await waitFor(() =>
    expect(uploadFile).toHaveBeenCalledWith(file, { kind: "session", id: "session-1" }),
  );
  expect(onAdded).toHaveBeenCalledWith(roomFile);
});
