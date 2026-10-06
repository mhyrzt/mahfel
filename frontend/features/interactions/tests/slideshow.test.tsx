import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";
import { ActivitiesPanel } from "../activities-panel";
import { poll, quizClosed, snapshot } from "../stories/fixtures";

describe("question slideshow", () => {
  test("shows one question per slide, bounds navigation, and keeps unsent choices", () => {
    const { container } = render(
      <ActivitiesPanel
        sessionId="session"
        locale="en"
        snapshot={{ ...snapshot, activities: [poll, quizClosed] }}
        busy={false}
        error={null}
        send={vi.fn(async () => true)}
        onRetry={vi.fn()}
      />,
    );
    expect(screen.getByRole("button", { name: "Previous" })).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent("Question 1 of 2");
    fireEvent.click(screen.getAllByRole("checkbox")[0]);
    fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(container.querySelectorAll("[data-activity-slide]:not([hidden])")).toHaveLength(1);
    expect(screen.getByRole("button", { name: "Next" })).toBeDisabled();
    const current = screen.getByRole("group", { name: "Question 2 of 2" });
    expect(within(current).getByText(quizClosed.question)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Vote" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Previous" }));
    expect(screen.getAllByRole("checkbox")[0]).toBeChecked();
  });
  test("restores the selected question and reports navigation for URL state", () => {
    const select = vi.fn();
    const props = {
      sessionId: "session",
      locale: "en" as const,
      snapshot: { ...snapshot, activities: [poll, quizClosed] },
      busy: false,
      error: null,
      send: vi.fn(async () => true),
      onRetry: vi.fn(),
      onSelectActivity: select,
    };
    const { rerender } = render(<ActivitiesPanel {...props} selectedActivityId={quizClosed.id} />);
    expect(screen.getByRole("status")).toHaveTextContent("Question 2 of 2");
    fireEvent.click(screen.getByRole("button", { name: "Previous" }));
    expect(select).toHaveBeenCalledWith(poll.id);
    rerender(<ActivitiesPanel {...props} selectedActivityId={poll.id} />);
    expect(screen.getByRole("status")).toHaveTextContent("Question 1 of 2");
  });
  test("opens an embedded composer and returns to the unfinished question on cancel", () => {
    render(
      <ActivitiesPanel
        sessionId="session"
        locale="en"
        snapshot={{ ...snapshot, activities: [poll, quizClosed] }}
        busy={false}
        error={null}
        send={vi.fn(async () => true)}
        onRetry={vi.fn()}
      />,
    );
    fireEvent.click(screen.getAllByRole("checkbox")[0]);
    fireEvent.click(screen.getByRole("button", { name: "Create" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Question" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Next" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Vote" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.getAllByRole("checkbox")[0]).toBeChecked();
    expect(screen.getByRole("status")).toHaveTextContent("Question 1 of 2");
  });
  test("Persian navigation localizes numbering and mirrors arrow direction", () => {
    const { container } = render(
      <ActivitiesPanel
        sessionId="session"
        locale="fa"
        snapshot={snapshot}
        busy={false}
        error={null}
        send={vi.fn(async () => true)}
        onRetry={vi.fn()}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("پرسش ۱ از ۲");
    expect(container.querySelector("section")).toHaveAttribute("dir", "rtl");
    expect(screen.getByRole("button", { name: "بعدی" }).querySelector("svg")).toHaveClass(
      "rtl:rotate-180",
    );
    fireEvent.click(screen.getByRole("button", { name: "بعدی" }));
    expect(screen.getByRole("status")).toHaveTextContent("پرسش ۲ از ۲");
  });
});
