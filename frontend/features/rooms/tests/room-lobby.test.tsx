import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, post } = vi.hoisted(() => ({ get: vi.fn(), post: vi.fn() }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get, post } }));

import { RoomLobby } from "../room-lobby";

beforeEach(() => {
  get.mockReset();
  post.mockReset();
});

test("a manager admits a room-link request from the room queue", async () => {
  get.mockResolvedValue({
    data: {
      entries: [
        { request_id: "request-1", actor_id: "actor-1", requested_at: "2026-09-25T00:00:00Z" },
      ],
    },
  });
  post.mockResolvedValue({});
  render(<RoomLobby roomId="room-1" locale="en" />);

  await screen.findByText("Participant acto");
  fireEvent.click(screen.getByRole("button", { name: /Admit/ }));

  await waitFor(() => expect(post).toHaveBeenCalledWith("/v1/rooms/room-1/lobby/actor-1/admit"));
});
