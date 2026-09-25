import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { listFiles, get, getFileDownloadUrl } = vi.hoisted(() => ({
  listFiles: vi.fn(),
  get: vi.fn(),
  getFileDownloadUrl: vi.fn(),
}));

vi.mock("@/features/files/client", () => ({ listFiles, getFileDownloadUrl }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get } }));

import { RoomSessions } from "../room-sessions";

const session = {
  id: "session-1",
  room_id: "room-1",
  title: "Weekly review",
  title_overridden: true,
  status: "live" as const,
  scheduled_start_at: "2026-09-25T09:30:00Z",
  scheduled_duration_minutes: 60,
  created_at: "2026-09-24T09:00:00Z",
  updated_at: "2026-09-25T09:00:00Z",
};

const file = (id: string) => ({
  id,
  file_name: `${id}.pdf`,
  context_kind: "session" as const,
  context_id: session.id,
  owner_id: "actor-1",
  mime: "application/pdf",
  size_bytes: 1024,
  checksum_sha256: "hash",
  status: "ready",
  created_at: "2026-09-24T09:00:00Z",
  updated_at: "2026-09-24T09:00:00Z",
});

beforeEach(() => {
  listFiles.mockReset();
  get.mockReset();
  getFileDownloadUrl.mockReset();
});

test("shows scheduling, live state, and unavailable playback", () => {
  render(<RoomSessions locale="en" sessions={[session]} />);
  const row = screen.getByTestId("room-session-session-1");
  expect(within(row).getByText("Ongoing")).toBeInTheDocument();
  expect(within(row).getByText("Weekly review")).toHaveAttribute("href", "/session/session-1");
  expect(within(row).getByText("60 min")).toBeInTheDocument();
  expect(
    within(row).getByRole("button", { name: "Playback is not available yet." }),
  ).toBeDisabled();
  expect(within(row).getByRole("link", { name: "Join session" })).toHaveAttribute(
    "href",
    "/session/session-1",
  );
  expect(within(row).getByTestId("session-schedule").querySelector("time")).toHaveAttribute(
    "datetime",
    session.scheduled_start_at,
  );
});

test("scheduled sessions keep the join label on the icon action", () => {
  render(<RoomSessions locale="en" sessions={[{ ...session, status: "scheduled" }]} />);
  expect(screen.getByRole("link", { name: "Join session" })).toHaveAttribute(
    "href",
    "/session/session-1",
  );
  expect(screen.queryByRole("link", { name: "View session" })).not.toBeInTheDocument();
});

test("uses Persian status and playback copy", () => {
  render(<RoomSessions locale="fa" sessions={[session]} />);
  expect(screen.getByText("در حال برگزاری")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "بازپخش هنوز در دسترس نیست." })).toBeDisabled();
});

test("expands session files and separates presented files from other attachments", async () => {
  listFiles.mockResolvedValue([file("presented"), file("handout")]);
  get.mockResolvedValue({ data: { file_ids: ["presented"] } });
  render(<RoomSessions locale="en" sessions={[session]} />);

  fireEvent.click(screen.getByRole("button", { name: "Session files" }));
  await waitFor(() =>
    expect(listFiles).toHaveBeenCalledWith({ kind: "session", id: "session-1" }, 100),
  );
  expect(await screen.findByText("Used in presentation")).toBeInTheDocument();
  expect(screen.getByText("Attached files")).toBeInTheDocument();
  expect(screen.getByTestId("attachment-presented")).toHaveTextContent("presented.pdf");
  expect(screen.getByTestId("attachment-handout")).toHaveTextContent("handout.pdf");
  expect(screen.getByRole("button", { name: /Session files/ })).toHaveAttribute(
    "aria-expanded",
    "true",
  );
});

test("includes a room file when the session presentation uses it", async () => {
  listFiles.mockImplementation(({ kind }: { kind: string }) =>
    Promise.resolve(kind === "session" ? [file("handout")] : [file("room-slide")]),
  );
  get.mockResolvedValue({ data: { file_ids: ["room-slide"] } });
  render(<RoomSessions locale="en" sessions={[session]} />);

  fireEvent.click(screen.getByRole("button", { name: "Session files" }));
  expect(await screen.findByTestId("attachment-room-slide")).toBeInTheDocument();
  expect(listFiles).toHaveBeenCalledWith({ kind: "room", id: "room-1" }, 100);
});
