import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, put, remove } = vi.hoisted(() => ({ get: vi.fn(), put: vi.fn(), remove: vi.fn() }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get, put, delete: remove } }));

import { MessageReactions } from "../components/message-reactions";

beforeEach(() => {
  get.mockReset();
  put.mockReset();
  remove.mockReset();
});

test("sets and removes a reaction using the supported emoji contract", async () => {
  get
    .mockResolvedValueOnce({ data: { counts: [], my_reaction: null } })
    .mockResolvedValueOnce({ data: { counts: [{ emoji: "👍", count: 1 }], my_reaction: "👍" } })
    .mockResolvedValueOnce({ data: { counts: [], my_reaction: null } });
  put.mockResolvedValue({});
  remove.mockResolvedValue({});
  render(<MessageReactions roomId="room-1" messageId="message-1" locale="en" />);

  fireEvent.click(screen.getByRole("button", { name: "React" }));
  await waitFor(() => expect(get).toHaveBeenCalledOnce());
  fireEvent.click(screen.getByRole("button", { name: "React 👍" }));
  await waitFor(() =>
    expect(put).toHaveBeenCalledWith("/v1/rooms/room-1/messages/message-1/reaction", {
      emoji: "👍",
    }),
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "React 👍" })).toHaveAttribute(
      "aria-pressed",
      "true",
    ),
  );
  fireEvent.click(screen.getByRole("button", { name: "React 👍" }));
  await waitFor(() =>
    expect(remove).toHaveBeenCalledWith("/v1/rooms/room-1/messages/message-1/reaction"),
  );
});
