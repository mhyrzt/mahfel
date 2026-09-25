import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, post, push, refresh } = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
  push: vi.fn(),
  refresh: vi.fn(),
}));

vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get, post } }));
vi.mock("next/navigation", () => ({ useRouter: () => ({ push, refresh }) }));

import { CreateSessionForm } from "../components/create-session-form";

const rooms = [
  {
    id: "room-1",
    title: "Team sync",
    status: "active",
    visibility: "invite_only" as const,
    admission: "open" as const,
    created_by: "owner-1",
  },
];

beforeEach(() => {
  post.mockReset();
  get.mockReset();
  push.mockReset();
  refresh.mockReset();
});

test("creates an ad-hoc session with inherited title", async () => {
  get.mockResolvedValue({ data: { can_manage_session: true } });
  post.mockResolvedValue({ data: { id: "session-1" } });
  render(<CreateSessionForm rooms={rooms} locale="en" />);

  await waitFor(() => expect(screen.getByRole("button", { name: "New session" })).toBeEnabled());
  fireEvent.click(screen.getByRole("button", { name: "New session" }));

  await waitFor(() => expect(push).toHaveBeenCalledWith("/session/session-1"));
  expect(post).toHaveBeenCalledWith("/v1/rooms/room-1/sessions", {
    title: null,
    scheduled_start_at: null,
    scheduled_duration_minutes: null,
  });
});

test("prevents a member from creating a session", async () => {
  get.mockResolvedValue({ data: { can_manage_session: false } });
  render(<CreateSessionForm rooms={rooms} locale="en" />);
  expect(
    await screen.findByText("Only a room manager can create sessions here."),
  ).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "New session" })).toBeDisabled();
  expect(post).not.toHaveBeenCalled();
});

test("does not offer scheduling when there is no room", () => {
  render(<CreateSessionForm rooms={[]} locale="en" />);
  expect(screen.getByText("Create a room before scheduling a session.")).toBeInTheDocument();
  expect(screen.queryByTestId("create-session-form")).not.toBeInTheDocument();
});
