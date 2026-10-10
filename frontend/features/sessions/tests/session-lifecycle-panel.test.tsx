import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import { AxiosError } from "axios";

const { get, post, refresh } = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
  refresh: vi.fn(),
}));

vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get, post } }));
vi.mock("next/navigation", () => ({ useRouter: () => ({ refresh }) }));

import { SessionLifecyclePanel } from "../components/session-lifecycle-panel";

const session = {
  id: "session-1",
  room_id: "room-1",
  status: "scheduled" as const,
  title: "Weekly review",
  title_overridden: true,
  created_at: "2026-09-25T00:00:00Z",
  updated_at: "2026-09-25T00:00:00Z",
};

beforeEach(() => {
  post.mockReset();
  get.mockReset();
  refresh.mockReset();
});

test("a conflicting live session is identified and linked when start returns 409", async () => {
  const conflict = new AxiosError("Conflict");
  Object.assign(conflict, { response: { status: 409 } });
  post.mockRejectedValue(conflict);
  get.mockResolvedValue({ data: { data: [{ ...session, id: "live-session", status: "live" }] } });
  render(<SessionLifecyclePanel session={session} canManage locale="en" />);

  fireEvent.click(screen.getByRole("button", { name: "Start session" }));

  expect(await screen.findByText(/Another session is already live/)).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Open the live session" })).toHaveAttribute(
    "href",
    "/session/live-session",
  );
});

test("a manager can start a scheduled session", async () => {
  post.mockResolvedValue({ data: { ...session, status: "live" } });
  render(<SessionLifecyclePanel session={session} canManage locale="en" />);

  fireEvent.click(screen.getByRole("button", { name: "Start session" }));

  await waitFor(() => expect(refresh).toHaveBeenCalled());
  expect(post).toHaveBeenCalledWith("/v1/sessions/session-1/start");
});

test("recording readiness failure leaves start available with a retry message", async () => {
  const unavailable = new AxiosError("Unavailable");
  Object.assign(unavailable, { response: { status: 503 } });
  post.mockRejectedValue(unavailable);
  render(<SessionLifecyclePanel session={session} canManage locale="en" />);
  fireEvent.click(screen.getByRole("button", { name: "Start session" }));
  expect(await screen.findByText(/Recording could not start/)).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Start session" })).toBeEnabled();
  expect(refresh).not.toHaveBeenCalled();
});

test("a member sees the schedule without management actions", () => {
  render(<SessionLifecyclePanel session={session} canManage={false} locale="en" />);

  expect(screen.getByText(/planned time does not prevent an early start/)).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Start session" })).not.toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Cancel session" })).not.toBeInTheDocument();
});

test("ending a live session requires confirmation", async () => {
  post.mockResolvedValue({ data: { ...session, status: "ended" } });
  render(<SessionLifecyclePanel session={{ ...session, status: "live" }} canManage locale="en" />);

  fireEvent.click(screen.getByRole("button", { name: "End session for everyone" }));
  expect(post).not.toHaveBeenCalled();
  const dialog = screen.getByRole("dialog", { name: "End session for everyone" });
  fireEvent.click(within(dialog).getByRole("button", { name: "End session for everyone" }));

  await waitFor(() => expect(post).toHaveBeenCalledWith("/v1/sessions/session-1/end"));
});
