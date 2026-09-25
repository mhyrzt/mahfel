import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { listFiles, uploadFile, getFileDownloadUrl } = vi.hoisted(() => ({
  listFiles: vi.fn(),
  uploadFile: vi.fn(),
  getFileDownloadUrl: vi.fn(),
}));

vi.mock("@/features/files/client", () => ({ listFiles, uploadFile, getFileDownloadUrl }));

import { MessageFiles } from "../components/message-files";

beforeEach(() => {
  listFiles.mockReset();
  uploadFile.mockReset();
  getFileDownloadUrl.mockReset();
});

test("loads message attachments on demand and uses the attachment card", async () => {
  listFiles.mockResolvedValue([
    { id: "file-1", file_name: "notes.pdf", mime: "application/pdf", size_bytes: 2048 },
  ]);
  render(<MessageFiles canAttach={false} locale="en" messageId="message-1" />);
  expect(listFiles).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Attachments" }));
  expect(await screen.findByTestId("attachment-file-1")).toHaveTextContent("notes.pdf");
  expect(listFiles).toHaveBeenCalledWith({ kind: "chat_message", id: "message-1" });
});

test("uploads a file against the existing message and refreshes attachments", async () => {
  uploadFile.mockResolvedValue({ id: "file-2" });
  listFiles.mockResolvedValue([
    { id: "file-2", file_name: "agenda.pdf", mime: "application/pdf", size_bytes: 1024 },
  ]);
  render(<MessageFiles canAttach locale="en" messageId="message-2" />);

  fireEvent.change(screen.getByLabelText("Attach file"), {
    target: { files: [new File(["agenda"], "agenda.pdf", { type: "application/pdf" })] },
  });
  await waitFor(() =>
    expect(uploadFile).toHaveBeenCalledWith(expect.any(File), {
      kind: "chat_message",
      id: "message-2",
    }),
  );
  expect(await screen.findByTestId("attachment-file-2")).toHaveTextContent("agenda.pdf");
});
