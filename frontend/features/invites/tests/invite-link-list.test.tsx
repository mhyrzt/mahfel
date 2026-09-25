import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";

import { InviteLinkList } from "../components/invite-link-list";

test("shows localized invite targets in Persian", () => {
  render(
    <InviteLinkList
      locale="fa"
      links={[
        {
          id: "room-link",
          code: "ROOM-CODE",
          expiresAt: "۱۴۰۵/۷/۳",
          status: "active",
          target: "room",
        },
        {
          id: "session-link",
          code: "SESSION-CODE",
          expiresAt: "۱۴۰۵/۷/۳",
          status: "active",
          target: "session",
        },
      ]}
    />,
  );

  expect(screen.getByText(/محفل · انقضا/)).toBeInTheDocument();
  expect(screen.getByText(/دیدار · انقضا/)).toBeInTheDocument();
});
