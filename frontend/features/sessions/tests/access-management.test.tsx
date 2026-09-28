import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, put } = vi.hoisted(() => ({ get: vi.fn(), put: vi.fn() }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get, put } }));

import { AccessManagement } from "../components/access-management";

beforeEach(() => {
  get.mockReset();
  put.mockReset();
  get.mockResolvedValue({
    data: {
      participants: [
        {
          actor_id: "manager",
          display_name: "Misato",
          can_publish_audio: true,
          can_publish_video: true,
          can_share_screen: true,
          can_present: true,
          can_edit: true,
          is_manager: true,
        },
        {
          actor_id: "member",
          display_name: "Shinji",
          can_publish_audio: true,
          can_publish_video: true,
          can_share_screen: true,
          can_present: false,
          can_edit: false,
          is_manager: false,
        },
      ],
    },
  });
  put.mockResolvedValue({});
});

test("a manager cannot change their own access and can grant presenting separately", async () => {
  render(<AccessManagement actorId="manager" sessionId="session" locale="en" />);

  const ownEdit = await screen.findByRole("checkbox", { name: "Misato: Edit" });
  expect(ownEdit).toBeDisabled();
  const present = screen.getByRole("checkbox", { name: "Shinji: Present" });
  fireEvent.click(present);

  await waitFor(() =>
    expect(put).toHaveBeenCalledWith("/v1/sessions/session/collaboration/grants/member", {
      can_present: true,
    }),
  );
  expect(screen.getByRole("checkbox", { name: "Shinji: Edit" })).not.toBeChecked();
});
