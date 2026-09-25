import { fireEvent, render, screen, within } from "@testing-library/react";
import { expect, test } from "vitest";

import { RoomDetailView } from "../room-detail";
import { RoomSettingsDialog } from "../room-settings-dialog";

const room = {
  id: "room-1",
  title: "Research methods",
  status: "active",
  visibility: "invite_only" as const,
  admission: "lobby" as const,
  created_by: "manager-1",
};

test("keeps manager controls in a tabbed dialog and omits conversation", () => {
  render(
    <RoomDetailView
      room={room}
      sessions={[]}
      locale="en"
      canManageSession={false}
      files={<p>Shared files</p>}
      settings={
        <RoomSettingsDialog
          locale="en"
          lobby={<p>Pending request</p>}
          admissionSettings={<p>Joining policy control</p>}
          invites={<p>Invite code control</p>}
        />
      }
    />,
  );

  const page = screen.getByTestId("room-detail");
  expect(within(page).getByTestId("room-sessions")).toBeInTheDocument();
  expect(within(page).getByText("Shared files")).toBeInTheDocument();
  expect(within(page).queryByText("Conversation")).not.toBeInTheDocument();
  expect(screen.queryByText("Pending request")).not.toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "Room settings" }));
  const dialog = screen.getByRole("dialog", { name: "Room settings" });
  expect(within(dialog).getByText("Pending request")).toBeInTheDocument();
  fireEvent.mouseDown(within(dialog).getByRole("tab", { name: "Admission" }), { button: 0 });
  expect(within(dialog).getByText("Joining policy control")).toBeInTheDocument();
  fireEvent.mouseDown(within(dialog).getByRole("tab", { name: "Invite links" }), { button: 0 });
  expect(within(dialog).getByText("Invite code control")).toBeInTheDocument();
  expect(within(page).queryByText("Invite code control")).not.toBeInTheDocument();
});

test("member room view has no management entry", () => {
  render(<RoomDetailView room={room} sessions={[]} locale="en" canManageSession={false} />);
  expect(screen.queryByRole("button", { name: "Room settings" })).not.toBeInTheDocument();
  expect(screen.getByTestId("room-sessions")).toBeInTheDocument();
});
