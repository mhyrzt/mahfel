import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, post } = vi.hoisted(() => ({ get: vi.fn(), post: vi.fn() }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get, post } }));
import { RecordingControls } from "../recording-controls";

beforeEach(() => {
  get.mockReset();
  post.mockReset();
});

function state(capabilities: string[], recordings: unknown[] = []) {
  get.mockImplementation((path: string) =>
    Promise.resolve({ data: path.endsWith("/capabilities") ? { capabilities } : recordings }),
  );
}

async function openMenu() {
  const trigger = screen.getByRole("button", { name: /Session recording|ضبط دیدار/ });
  trigger.focus();
  fireEvent.keyDown(trigger, { key: "Enter" });
  return screen.findByRole("menu");
}

test("members see recording state and capture warnings without controls", async () => {
  state(
    [],
    [
      {
        id: "r",
        session_id: "s",
        status: "recording",
        revision: 1,
        capture_generation: 1,
        started_at: "2026-09-28T12:00:00Z",
        stopped_at: null,
        warning: "storage_unavailable",
        partial: true,
      },
    ],
  );
  render(<RecordingControls sessionId="s" locale="en" />);
  await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Recording has gaps"));
  expect(screen.getByRole("alert")).toHaveTextContent("live session can continue");
  await openMenu();
  expect(screen.queryByRole("menuitem")).not.toBeInTheDocument();
});

test("a failed command retries with the same id to prevent duplicate recording", async () => {
  state(["manage_recording"]);
  post.mockRejectedValueOnce(new Error("lost response")).mockResolvedValueOnce({
    data: { id: "r", status: "starting", revision: 1, started_at: "2026-09-28T12:00:00Z" },
  });
  render(<RecordingControls sessionId="s" locale="en" />);
  await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Recording is off"));
  await openMenu();
  fireEvent.click(screen.getByRole("menuitem", { name: "Start recording" }));
  await screen.findByRole("alert");
  fireEvent.click(screen.getByRole("menuitem", { name: "Start recording" }));
  await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Starting recording"));
  expect(post.mock.calls[0]?.[1]).toEqual(post.mock.calls[1]?.[1]);
  expect(screen.getByRole("menuitem", { name: "Stop recording" })).toBeInTheDocument();
});

test("unknown status is visible when the API is unavailable", async () => {
  get.mockRejectedValue(new Error("offline"));
  render(<RecordingControls sessionId="s" locale="fa" />);
  expect(await screen.findByRole("alert")).toHaveTextContent("بررسی وضعیت ضبط ممکن نشد");
  await openMenu();
  expect(screen.queryByRole("menuitem")).not.toBeInTheDocument();
});

test("an admitted participant sees recording notice when capability lookup fails", async () => {
  get.mockImplementation((path: string) =>
    path.endsWith("/capabilities")
      ? Promise.reject(new Error("capabilities unavailable"))
      : Promise.resolve({
          data: [
            {
              id: "r",
              status: "recording",
              revision: 1,
              started_at: "2026-09-28T12:00:00Z",
              partial: false,
            },
          ],
        }),
  );
  render(<RecordingControls sessionId="s" locale="en" />);
  await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Recording"));
  await openMenu();
  expect(screen.getByText(/Media, shared workspace and session chat/)).toBeInTheDocument();
  expect(screen.queryByRole("menuitem")).not.toBeInTheDocument();
});

test("recording menu is keyboard reachable and Escape returns focus to its indicator", async () => {
  state(["manage_recording"]);
  render(<RecordingControls sessionId="s" locale="en" />);
  await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Recording is off"));
  const trigger = screen.getByRole("button", { name: /Session recording/ });
  trigger.focus();
  fireEvent.keyDown(trigger, { key: "Enter" });
  await screen.findByRole("menu");
  const start = screen.getByRole("menuitem", { name: "Start recording" });
  await waitFor(() => expect(start).toHaveFocus());
  fireEvent.keyDown(start, { key: "Escape" });
  await waitFor(() => expect(screen.queryByRole("menu")).not.toBeInTheDocument());
  expect(trigger).toHaveFocus();
});
