import { render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { getFileDownloadUrl } = vi.hoisted(() => ({ getFileDownloadUrl: vi.fn() }));

vi.mock("@/features/files/client", () => ({ getFileDownloadUrl }));

import { MessageAttachments } from "../components/message-attachments";

const base = {
  owner_id: "actor-1",
  context_kind: "chat_message" as const,
  context_id: "message-1",
  status: "ready",
  checksum_sha256: "hash",
  created_at: "2026-09-25T00:00:00Z",
  updated_at: "2026-09-25T00:00:00Z",
};

beforeEach(() => {
  getFileDownloadUrl.mockReset();
  getFileDownloadUrl.mockResolvedValue("https://storage.test/file");
});

test("renders multiple media files as an album grid", () => {
  render(
    <MessageAttachments
      locale="en"
      files={[
        { ...base, id: "img-1", file_name: "one.png", mime: "image/png", size_bytes: 1024 },
        { ...base, id: "img-2", file_name: "two.png", mime: "image/png", size_bytes: 2048 },
        { ...base, id: "doc-1", file_name: "notes.pdf", mime: "application/pdf", size_bytes: 512 },
      ]}
    />,
  );

  expect(screen.getByTestId("message-media-grid")).toHaveClass("grid-cols-2");
  expect(screen.getByRole("button", { name: "one.png" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "two.png" })).toBeInTheDocument();
  expect(screen.getByTestId("attachment-doc-1")).toHaveTextContent("notes.pdf");
});

test("renders a video attachment with a play affordance", () => {
  render(
    <MessageAttachments
      locale="en"
      files={[{ ...base, id: "vid-1", file_name: "clip.mp4", mime: "video/mp4", size_bytes: 9000 }]}
    />,
  );
  expect(screen.getByRole("button", { name: "clip.mp4" })).toBeInTheDocument();
  expect(screen.getByTestId("message-media-grid")).toHaveClass("grid-cols-1");
});

test("renders nothing when there are no attachments", () => {
  render(<MessageAttachments files={[]} locale="en" />);
  expect(screen.queryByTestId("message-attachments")).not.toBeInTheDocument();
});
