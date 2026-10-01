import { render, screen, fireEvent } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { RecordingListView } from "../recording-list-view";
import { RecordingList } from "../recording-list";
import { browserApiClient } from "@/lib/api/browser";

vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get: vi.fn() } }));

const item = {
  id: "recording",
  started_at: "2026-09-29T08:00:00Z",
  status: "ready",
  partial: true,
  export: {
    id: "export",
    recording_id: "recording",
    status: "ready",
    progress: 100,
    duration_ms: 2000,
    size_bytes: 40000,
    failure_code: null,
    partial: true,
  },
};
test("ready exports can be downloaded and show partial coverage", () => {
  const download = vi.fn();
  render(<RecordingListView items={[item]} locale="en" onDownload={download} />);
  fireEvent.click(screen.getByRole("button", { name: "Download MP4" }));
  expect(download).toHaveBeenCalledWith("recording");
  expect(screen.getByText("Some parts of this recording are missing.")).toBeInTheDocument();
});
test("only managers see export retry", () => {
  const failed = { ...item, export: { ...item.export, status: "failed" } };
  const view = render(<RecordingListView items={[failed]} locale="en" />);
  expect(screen.queryByRole("button", { name: "Retry export" })).not.toBeInTheDocument();
  view.rerender(<RecordingListView items={[failed]} locale="en" canManage />);
  expect(screen.getByRole("button", { name: "Retry export" })).toBeInTheDocument();
});
test("ready recordings can be played by members", () => {
  const play = vi.fn();
  render(<RecordingListView items={[item]} locale="en" onPlay={play} />);
  fireEvent.click(screen.getByRole("button", { name: "Play recording" }));
  expect(play).toHaveBeenCalledWith("recording");
});
test("interactive replay is available while MP4 export is still queued", () => {
  render(<RecordingListView items={[{ ...item, export: null }]} locale="en" />);
  expect(screen.getByRole("button", { name: "Play recording" })).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Download MP4" })).not.toBeInTheDocument();
});
test("an unavailable export endpoint does not hide a ready replay", async () => {
  vi.mocked(browserApiClient.get)
    .mockResolvedValueOnce({ data: [item] })
    .mockRejectedValueOnce(new Error("export unavailable"));
  render(<RecordingList sessionId="session" locale="en" />);
  fireEvent.click(screen.getByRole("button", { name: "Recordings" }));
  expect(await screen.findByRole("button", { name: "Play recording" })).toBeInTheDocument();
});
test("managers can retry failed replay preparation", () => {
  const retry = vi.fn();
  const failed = {
    ...item,
    status: "finalizing",
    warning: "playback_processing_failed",
    export: null,
  };
  const view = render(<RecordingListView items={[failed]} locale="en" onRetryPlayback={retry} />);
  expect(
    screen.queryByRole("button", { name: "Retry replay preparation" }),
  ).not.toBeInTheDocument();
  view.rerender(
    <RecordingListView items={[failed]} locale="en" canManage onRetryPlayback={retry} />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Retry replay preparation" }));
  expect(retry).toHaveBeenCalledWith("recording");
});
test("only managers can delete recordings", () => {
  const remove = vi.fn();
  const view = render(<RecordingListView items={[item]} locale="en" onRemove={remove} />);
  expect(screen.queryByRole("button", { name: "Delete recording" })).not.toBeInTheDocument();
  view.rerender(<RecordingListView items={[item]} locale="en" canManage onRemove={remove} />);
  fireEvent.click(screen.getByRole("button", { name: "Delete recording" }));
  expect(remove).toHaveBeenCalledWith("recording");
});
