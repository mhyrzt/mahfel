import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get } = vi.hoisted(() => ({ get: vi.fn() }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get } }));

import { WorkspaceBrowserDialog } from "../components/workspace-browser-dialog";

const room = {
  id: "room-1",
  title: "Design review",
  visibility: "invite_only",
  admission: "open",
  created_by: "manager-1",
  status: "active",
};

beforeEach(() => get.mockReset());

test("browses rooms without an index route", async () => {
  const onOpenChange = vi.fn();
  get.mockResolvedValue({ data: { data: [room] } });
  render(<WorkspaceBrowserDialog kind="rooms" locale="en" open onOpenChange={onOpenChange} />);
  const link = await screen.findByRole("link", { name: "Design review" });
  expect(link).toHaveAttribute("href", "/room/room-1");
  fireEvent.click(link);
  expect(onOpenChange).toHaveBeenCalledWith(false);
});

test("browses sessions within a selected room", async () => {
  get.mockImplementation((path: string) =>
    path === "/v1/rooms"
      ? Promise.resolve({ data: { data: [room] } })
      : Promise.resolve({ data: [{ id: "session-1", title: "Weekly review" }] }),
  );
  render(
    <WorkspaceBrowserDialog kind="sessions" locale="en" open onOpenChange={() => undefined} />,
  );
  await waitFor(() => expect(get).toHaveBeenCalledWith("/v1/rooms/room-1/sessions"));
  expect(await screen.findByRole("link", { name: "Weekly review" })).toHaveAttribute(
    "href",
    "/session/session-1",
  );
});

test("can reach rooms beyond the first page", async () => {
  get.mockImplementation((_path: string, options?: { params: { page: number } }) =>
    Promise.resolve({
      data: {
        data:
          (options?.params.page ?? 1) === 1
            ? Array.from({ length: 100 }, (_, index) => ({
                ...room,
                id: `room-${index + 1}`,
                title: `Room ${index + 1}`,
              }))
            : [{ ...room, id: "room-101", title: "Room 101" }],
      },
    }),
  );
  render(<WorkspaceBrowserDialog kind="rooms" locale="en" open onOpenChange={() => undefined} />);
  fireEvent.click(await screen.findByRole("button", { name: "Load more rooms" }));
  expect(await screen.findByRole("link", { name: "Room 101" })).toHaveAttribute(
    "href",
    "/room/room-101",
  );
});
