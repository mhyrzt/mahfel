import { render, screen, within } from "@testing-library/react";
import { expect, test } from "vitest";

import { ParticipantRoster } from "../components/participant-roster";

test("moves a speaking participant first and distinguishes a muted microphone", () => {
  render(
    <ParticipantRoster
      participants={[
        { id: "quiet", name: "Quiet member", initials: "QM", role: "member", muted: true },
        { id: "speaker", name: "Speaking member", initials: "SM", role: "member", speaking: true },
      ]}
    />,
  );
  const rows = within(screen.getByRole("list")).getAllByRole("listitem");
  expect(rows[0]).toHaveTextContent("Speaking member");
  expect(within(rows[0]).getByLabelText("Speaking")).toBeInTheDocument();
  expect(within(rows[1]).getByLabelText("Muted")).toBeInTheDocument();
});

test("orders by role, then speaking status, then name", () => {
  render(
    <ParticipantRoster
      participants={[
        { id: "member-z", name: "Zoe member", initials: "ZM", role: "member", speaking: true },
        { id: "manager-z", name: "Zoe manager", initials: "ZM", role: "manager" },
        { id: "moderator", name: "Quiet moderator", initials: "QM", role: "moderator" },
        { id: "member-a", name: "Alex member", initials: "AM", role: "member", speaking: true },
        { id: "manager-a", name: "Alex manager", initials: "AM", role: "manager" },
        { id: "member-quiet", name: "Aaron member", initials: "AM", role: "member" },
        {
          id: "manager-speaking",
          name: "Zara manager",
          initials: "ZM",
          role: "manager",
          speaking: true,
        },
      ]}
    />,
  );
  const rows = within(screen.getByRole("list")).getAllByRole("listitem");
  expect(rows.map((row) => within(row).getByText(/member|manager|moderator/).textContent)).toEqual([
    "Zara manager",
    "Alex manager",
    "Zoe manager",
    "Quiet moderator",
    "Alex member",
    "Zoe member",
    "Aaron member",
  ]);
});
