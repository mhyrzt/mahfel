import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, post, patch, remove } = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
  patch: vi.fn(),
  remove: vi.fn(),
}));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get, post, patch, delete: remove } }));

import { RoomMembers } from "../room-members";

const manager = { user_id: "00000000-0000-0000-0000-000000000001", role: "manager" as const };
const member = { user_id: "00000000-0000-0000-0000-000000000002", role: "member" as const };

beforeEach(() => {
  get.mockReset();
  post.mockReset();
  patch.mockReset();
  remove.mockReset();
});

test("loads members and changes their roles", async () => {
  get.mockResolvedValue({ data: { members: [manager, member] } });
  patch.mockResolvedValue({ data: { ...member, role: "manager" } });
  render(<RoomMembers roomId="room-1" locale="en" actorId={manager.user_id} />);

  const row = await screen.findByTestId(`room-member-${member.user_id}`);
  expect(get).toHaveBeenCalledWith("/v1/rooms/room-1/members");
  fireEvent.change(within(row).getByRole("combobox"), { target: { value: "manager" } });
  await waitFor(() =>
    expect(patch).toHaveBeenCalledWith(`/v1/rooms/room-1/members/${member.user_id}`, {
      user_id: member.user_id,
      role: "manager",
    }),
  );
  expect(within(row).getByRole("combobox")).toHaveValue("manager");
});

test("adds and removes a member while protecting the last manager", async () => {
  post.mockResolvedValue({ data: member });
  remove.mockResolvedValue({});
  render(<RoomMembers roomId="room-1" locale="en" initialMembers={[manager]} />);

  const managerRow = screen.getByTestId(`room-member-${manager.user_id}`);
  expect(within(managerRow).getByRole("combobox")).toBeDisabled();
  expect(
    within(managerRow).getByRole("button", { name: `Remove member: ${manager.user_id}` }),
  ).toBeDisabled();

  fireEvent.change(screen.getByRole("textbox", { name: "User ID" }), {
    target: { value: member.user_id },
  });
  fireEvent.click(screen.getByRole("button", { name: "Add member" }));
  await waitFor(() =>
    expect(post).toHaveBeenCalledWith("/v1/rooms/room-1/members", {
      user_id: member.user_id,
      role: "member",
    }),
  );
  const memberRow = screen.getByTestId(`room-member-${member.user_id}`);
  fireEvent.click(
    within(memberRow).getByRole("button", { name: `Remove member: ${member.user_id}` }),
  );
  fireEvent.click(within(memberRow).getByRole("button", { name: "Confirm remove" }));
  await waitFor(() =>
    expect(remove).toHaveBeenCalledWith(`/v1/rooms/room-1/members/${member.user_id}`),
  );
  expect(screen.queryByTestId(`room-member-${member.user_id}`)).not.toBeInTheDocument();
});
