import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get } = vi.hoisted(() => ({ get: vi.fn() }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get } }));
vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), refresh: vi.fn() }) }));

import { WorkspaceBrowserMenu } from "../components/workspace-browser-menu";

const room = {
  id: "room-1",
  title: "Design review",
  visibility: "invite_only",
  admission: "open",
  created_by: "manager-1",
  status: "active",
};

beforeEach(() => {
  get.mockReset();
});

test("browses rooms without an index route", async () => {
  const onOpenChange = vi.fn();
  get.mockResolvedValue({ data: { data: [room] } });
  render(
    <WorkspaceBrowserMenu kind="rooms" locale="en" open onOpenChange={onOpenChange}>
      <button>Rooms</button>
    </WorkspaceBrowserMenu>,
  );
  const link = await screen.findByRole("menuitem", { name: "Design review" });
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
    <WorkspaceBrowserMenu kind="sessions" locale="en" open onOpenChange={() => undefined}>
      <button>Browse</button>
    </WorkspaceBrowserMenu>,
  );
  await waitFor(() => expect(get).toHaveBeenCalledWith("/v1/rooms/room-1/sessions"));
  expect(await screen.findByRole("menuitem", { name: "Weekly review" })).toHaveAttribute(
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
  render(
    <WorkspaceBrowserMenu kind="rooms" locale="en" open onOpenChange={() => undefined}>
      <button>Browse</button>
    </WorkspaceBrowserMenu>,
  );
  fireEvent.click(await screen.findByRole("menuitem", { name: "Load more rooms" }));
  expect(await screen.findByRole("menuitem", { name: "Room 101" })).toHaveAttribute(
    "href",
    "/room/room-101",
  );
});

test.each([true, false])("session creation follows room permission %s", async (canManage) => {
  get.mockImplementation((path: string) => {
    if (path === "/v1/rooms") {
      return Promise.resolve({ data: { data: [room] } });
    }
    if (path.endsWith("/membership")) {
      return Promise.resolve({ data: { can_manage_session: canManage } });
    }
    return Promise.resolve({ data: [] });
  });
  render(
    <WorkspaceBrowserMenu kind="sessions" locale="en" open onOpenChange={() => undefined}>
      <button>Browse</button>
    </WorkspaceBrowserMenu>,
  );
  await waitFor(() => expect(get).toHaveBeenCalledWith("/v1/rooms/room-1/membership"));
  await screen.findByText("No sessions yet.");
  if (canManage) {
    expect(await screen.findByRole("menuitem", { name: "Create session" })).toBeVisible();
  } else {
    expect(screen.queryByRole("menuitem", { name: "Create session" })).not.toBeInTheDocument();
  }
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
});

test("empty room menu still offers room creation", async () => {
  get.mockResolvedValue({ data: { data: [] } });
  render(
    <WorkspaceBrowserMenu kind="rooms" locale="en" open onOpenChange={() => undefined}>
      <button>Rooms</button>
    </WorkspaceBrowserMenu>,
  );
  expect(await screen.findByRole("menuitem", { name: "Create room" })).toBeVisible();
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
});

test("failed authorization hides session creation", async () => {
  get.mockImplementation((path: string) =>
    path.endsWith("/membership")
      ? Promise.reject(new Error("Forbidden"))
      : Promise.resolve({ data: path === "/v1/rooms" ? { data: [room] } : [] }),
  );
  render(
    <WorkspaceBrowserMenu kind="sessions" locale="fa" open onOpenChange={() => undefined}>
      <button>Sessions</button>
    </WorkspaceBrowserMenu>,
  );
  await waitFor(() => expect(get).toHaveBeenCalledWith("/v1/rooms/room-1/membership"));
  await waitFor(() =>
    expect(screen.queryByRole("menuitem", { name: "ایجاد دیدار" })).not.toBeInTheDocument(),
  );
  expect(screen.queryByRole("menuitem", { name: "ایجاد دیدار" })).not.toBeInTheDocument();
});

test("session create action opens the existing form with the selected room", async () => {
  get.mockImplementation((path: string) => {
    if (path === "/v1/rooms") {
      return Promise.resolve({ data: { data: [room] } });
    }
    if (path.endsWith("/membership")) {
      return Promise.resolve({ data: { can_manage_session: true } });
    }
    return Promise.resolve({ data: [] });
  });
  render(
    <WorkspaceBrowserMenu kind="sessions" locale="en" open onOpenChange={() => undefined}>
      <button>Sessions</button>
    </WorkspaceBrowserMenu>,
  );
  fireEvent.click(await screen.findByRole("menuitem", { name: "Create session" }));
  expect(await screen.findByRole("dialog")).toBeVisible();
  expect(screen.getByRole("combobox", { name: "Room" })).toHaveValue("room-1");
  await waitFor(() => expect(screen.getByRole("button", { name: "New session" })).toBeEnabled());
});
