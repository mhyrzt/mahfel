import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { post } = vi.hoisted(() => ({ post: vi.fn() }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { post, get: vi.fn() } }));

import { JoinByCodePanel } from "../components/join-by-code-panel";

beforeEach(() => post.mockReset());

test("redeems a code in the request body and opens an existing membership", async () => {
  post.mockResolvedValue({
    data: { outcome: "already_member", request_id: null, target_kind: "room", target_id: "room-1" },
  });
  render(<JoinByCodePanel locale="en" />);
  fireEvent.change(screen.getByRole("textbox", { name: "Invite code" }), {
    target: { value: "ABC-DEFG-HIJ" },
  });
  await act(async () => fireEvent.click(screen.getByRole("button", { name: "Continue" })));

  await waitFor(() =>
    expect(screen.getByRole("link", { name: "Open destination" })).toHaveAttribute(
      "href",
      "/room/room-1",
    ),
  );
  expect(post).toHaveBeenCalledWith("/v1/invite-links/redeem", { code: "ABC-DEFG-HIJ" });
});

test("shows an unusable code state without exposing a destination", () => {
  render(<JoinByCodePanel locale="en" status="unavailable" />);
  expect(screen.getByText("Invite unavailable")).toBeInTheDocument();
  expect(screen.queryByRole("link", { name: "Open destination" })).not.toBeInTheDocument();
});
