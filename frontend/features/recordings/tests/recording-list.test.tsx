import { render, screen, fireEvent } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { RecordingListView } from "../recording-list-view";

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
test("only managers can delete recordings", () => {
  const remove = vi.fn();
  const view = render(<RecordingListView items={[item]} locale="en" onRemove={remove} />);
  expect(screen.queryByRole("button", { name: "Delete recording" })).not.toBeInTheDocument();
  view.rerender(<RecordingListView items={[item]} locale="en" canManage onRemove={remove} />);
  fireEvent.click(screen.getByRole("button", { name: "Delete recording" }));
  expect(remove).toHaveBeenCalledWith("recording");
});
