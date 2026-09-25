import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";

const { put } = vi.hoisted(() => ({ put: vi.fn() }));
vi.mock("@/lib/api/browser", () => ({ browserApiClient: { put } }));
const { refresh } = vi.hoisted(() => ({ refresh: vi.fn() }));
vi.mock("next/navigation", () => ({ useRouter: () => ({ refresh }) }));

import { RoomAdmissionSettings } from "../room-admission-settings";

test("a manager can change the room's joining policy", async () => {
  put.mockResolvedValue({ data: { policy: "lobby" } });
  render(<RoomAdmissionSettings roomId="room-1" initialPolicy="open" locale="en" />);
  fireEvent.change(screen.getByRole("combobox", { name: "Joining policy" }), {
    target: { value: "lobby" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(put).toHaveBeenCalledWith("/v1/rooms/room-1/admission", { policy: "lobby" }),
  );
  expect(screen.getByRole("combobox", { name: "Joining policy" })).toHaveValue("lobby");
  expect(refresh).toHaveBeenCalledOnce();
});
