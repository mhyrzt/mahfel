import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";

import { ParticipationSummary } from "../components/participation-summary";

test("shows durable connected time after a session ends", () => {
  render(
    <ParticipationSummary
      locale="en"
      data={{
        session_id: "session-1",
        participants: [
          {
            participant_id: "participant-1",
            actor_id: "actor-123456",
            connected_seconds: 150,
            corrected_intervals: 0,
          },
        ],
      }}
    />,
  );
  expect(screen.getByTestId("participation-summary")).toHaveTextContent("Participant actor-12");
  expect(screen.getByTestId("participation-summary")).toHaveTextContent("2.5 min");
});
