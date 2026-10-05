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
  fireEvent.click(screen.getByRole("switch", { name: "Lobby admission" }));
  await waitFor(() =>
    expect(put).toHaveBeenCalledWith("/v1/rooms/room-1/admission", { policy: "lobby" }),
  );
  await waitFor(() => {
    expect(screen.getByRole("switch", { name: "Lobby admission" })).toHaveAttribute(
      "aria-checked",
      "true",
    );
    expect(refresh).toHaveBeenCalledOnce();
  });
});

test("a failed admission change keeps the saved policy and re-enables the switch", async () => {
  put.mockRejectedValue(new Error("Unavailable"));
  render(<RoomAdmissionSettings roomId="room-1" initialPolicy="open" locale="en" />);
  const toggle = screen.getByRole("switch", { name: "Lobby admission" });
  fireEvent.click(toggle);
  expect(toggle).toBeDisabled();
  await screen.findByRole("alert");
  expect(toggle).not.toBeDisabled();
  expect(toggle).not.toBeChecked();
});
