import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get } = vi.hoisted(() => ({ get: vi.fn() }));

vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get } }));

import { SessionParticipantCount } from "../session-participant-count";

beforeEach(() => {
  get.mockReset();
});

test("shows the live participant count for an ongoing session", async () => {
  get.mockResolvedValue({ data: [{ id: "p1" }, { id: "p2" }] });
  render(<SessionParticipantCount live locale="en" sessionId="session-1" />);
  expect(await screen.findByTestId("session-participants-session-1")).toHaveTextContent(
    "2 in session",
  );
});

test("stays hidden for a session that is not live", () => {
  render(<SessionParticipantCount live={false} locale="en" sessionId="session-1" />);
  expect(screen.queryByTestId("session-participants-session-1")).not.toBeInTheDocument();
  expect(get).not.toHaveBeenCalled();
});

test("hides the count when the roster cannot be loaded", async () => {
  get.mockRejectedValue(new Error("denied"));
  render(<SessionParticipantCount live locale="en" sessionId="session-2" />);
  await waitFor(() => expect(get).toHaveBeenCalled());
  expect(screen.queryByTestId("session-participants-session-2")).not.toBeInTheDocument();
});
