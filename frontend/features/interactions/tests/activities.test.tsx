import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";
import { ActivityCard } from "../activity-card";
import { ActivityComposer } from "../activity-composer";
import { HandQueue } from "../hand-queue";
import { poll, pollResults, quiz, quizClosed, snapshot } from "../stories/fixtures";

const common = {
  sessionId: "session",
  locale: "en" as const,
  busy: false,
  canManage: false,
  canRespond: true,
  hasOpenActivity: true,
  onEdit: vi.fn(),
};
describe("session activities", () => {
  test("multiple-choice Questionnaire submits one selected set", async () => {
    const send = vi.fn(async () => true);
    render(<ActivityCard {...common} activity={poll} send={send} />);
    const choices = screen.getAllByRole("checkbox");
    fireEvent.click(choices[0]);
    fireEvent.click(choices[1]);
    fireEvent.click(screen.getByRole("button", { name: "Vote" }));
    await waitFor(() =>
      expect(send).toHaveBeenCalledWith({
        action: "vote",
        id: poll.id,
        selected: [0, 1],
        expected_version: 0,
      }),
    );
  });
  test("open quiz displays submitted choices without correctness or result bars", () => {
    const { container } = render(
      <ActivityCard {...common} activity={quiz} send={vi.fn(async () => true)} />,
    );
    expect(screen.getByText("Answer submitted")).toBeInTheDocument();
    expect(
      screen.getByText("Results will appear when the host closes the quiz."),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Change vote" })).not.toBeInTheDocument();
    expect(container.querySelector("[data-result-bar]")).toBeNull();
    expect(screen.queryByText("Incorrect")).not.toBeInTheDocument();
  });
  test("single-choice Questionnaire uses mutually exclusive radios", () => {
    render(
      <ActivityCard
        {...common}
        activity={{ ...poll, multiple: false }}
        send={vi.fn(async () => true)}
      />,
    );
    const choices = screen.getAllByRole("radio");
    fireEvent.click(choices[0]);
    fireEvent.click(choices[1]);
    expect(choices[0]).not.toBeChecked();
    expect(choices[1]).toBeChecked();
  });
  test("Persian voting and result bars preserve reading and fill direction", () => {
    const { container, rerender } = render(
      <ActivityCard {...common} locale="fa" activity={poll} send={vi.fn(async () => true)} />,
    );
    expect(container.querySelector("form")).toHaveAttribute("dir", "rtl");
    rerender(
      <ActivityCard
        {...common}
        locale="fa"
        activity={pollResults}
        send={vi.fn(async () => true)}
      />,
    );
    expect(container.querySelector("[data-result-bar]")).toHaveClass("rtl:origin-right");
    expect(
      screen.getByText(
        new Intl.NumberFormat("fa", { style: "percent", maximumFractionDigits: 0 }).format(0.8),
      ),
    ).toBeInTheDocument();
  });
  test("closed quiz marks the correct set and the participant's incorrect response", () => {
    const { container } = render(
      <ActivityCard {...common} activity={quizClosed} send={vi.fn(async () => true)} />,
    );
    expect(screen.getByText("Incorrect")).toBeInTheDocument();
    expect(screen.getByLabelText("Closed")).toHaveAttribute("title", "Closed");
    expect(screen.getAllByText("Correct")).toHaveLength(2);
    expect(container.querySelectorAll("[data-result-bar]")).toHaveLength(3);
    expect(screen.queryByRole("button", { name: "Submit answer" })).not.toBeInTheDocument();
  });
  test("poll bars use respondent percentages and retain vote version for retraction", () => {
    const send = vi.fn(async () => true);
    render(<ActivityCard {...common} activity={pollResults} send={send} />);
    expect(screen.getByText("80%")).toBeInTheDocument();
    expect(screen.getByText("60%")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "View voters" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Retract vote" }));
    expect(send).toHaveBeenCalledWith({ action: "retract", id: poll.id, expected_version: 1 });
  });
  test("creator builds a quiz with multiple correct answers", async () => {
    const send = vi.fn(async () => true);
    render(<ActivityComposer locale="en" onClose={vi.fn()} busy={false} send={send} />);
    fireEvent.click(screen.getByRole("radio", { name: "Quiz" }));
    fireEvent.change(screen.getByLabelText("Question"), { target: { value: "Prime numbers?" } });
    fireEvent.change(screen.getByLabelText("Option 1"), { target: { value: "2" } });
    fireEvent.change(screen.getByLabelText("Option 2"), { target: { value: "3" } });
    fireEvent.click(screen.getByRole("switch", { name: "Allow multiple answers" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Correct option 1" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Correct option 2" }));
    fireEvent.click(screen.getByRole("button", { name: "Save draft" }));
    await waitFor(() =>
      expect(send).toHaveBeenCalledWith(
        expect.objectContaining({
          action: "save_draft",
          definition: expect.objectContaining({
            kind: "quiz",
            correct_options: [0, 1],
            multiple: true,
            anonymous: true,
          }),
        }),
      ),
    );
  });
  test("host lowers a specific queue generation without changing media", () => {
    const send = vi.fn(async () => true);
    render(
      <HandQueue
        hands={snapshot.hands}
        version={4}
        canModerate
        busy={false}
        locale="en"
        send={send}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Lower hand: سارا احمدی" }));
    expect(send).toHaveBeenCalledWith({
      action: "lower_hand",
      participant_id: "participant-1",
      expected_order: 1,
    });
    fireEvent.click(screen.getByRole("button", { name: "Lower all hands" }));
    expect(send).toHaveBeenCalledWith({ action: "clear_hands", expected_version: 4 });
  });
});
