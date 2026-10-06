import { fireEvent, render, screen } from "@testing-library/react";
import { expect, test } from "vitest";

import { SessionStage } from "../components/session-stage";

test("moderator has no action menu for their own tile", () => {
  render(
    <SessionStage
      title="Practice"
      status="live"
      currentActorId="misato"
      currentParticipantId="self"
      participantActions={["mute", "remove", "block"]}
      participants={[
        { id: "self", actorId: "misato", name: "Misato", initials: "M", role: "manager" },
        { id: "other", actorId: "shinji", name: "Shinji", initials: "S", role: "member" },
      ]}
    />,
  );
  expect(screen.queryByRole("button", { name: "Actions for Misato" })).not.toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Actions for Shinji" })).toBeInTheDocument();
});

test("keeps an unfinished answer when switching away from the question view", () => {
  const props = {
    title: "Practice",
    status: "live" as const,
    participants: [],
    activities: <input aria-label="Draft answer" />,
  };
  const { rerender } = render(<SessionStage {...props} activeSurface="activities" />);
  fireEvent.change(screen.getByRole("textbox", { name: "Draft answer" }), {
    target: { value: "My choice" },
  });
  rerender(<SessionStage {...props} activeSurface="participants" />);
  expect(screen.queryByRole("textbox", { name: "Draft answer" })).not.toBeInTheDocument();
  rerender(<SessionStage {...props} activeSurface="activities" />);
  expect(screen.getByRole("textbox", { name: "Draft answer" })).toHaveValue("My choice");
});
