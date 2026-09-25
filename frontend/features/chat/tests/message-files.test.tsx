import { fireEvent, render, screen } from "@testing-library/react";
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
