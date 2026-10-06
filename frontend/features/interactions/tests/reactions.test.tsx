import { act, fireEvent, render, renderHook, screen } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { TooltipProvider } from "@/components/ui/tooltip";
import { SessionStage } from "@/features/sessions/components/session-stage";
import { HandControl } from "../hand-control";
import { ReactionOverlay, type LiveReaction } from "../reaction-overlay";
import { useSessionReactions } from "../use-session-reactions";

vi.mock("@/lib/api/browser", () => ({ browserApiClient: { post: vi.fn(async () => ({})) } }));

function reaction(eventId: string): LiveReaction {
  return {
    v: 1,
    type: "session_reaction",
    session_id: "session",
    event_id: eventId,
    participant_id: "participant",
    username: "سارا Ahmed",
    emoji: "👍",
  };
}

describe("session reactions and activity surface", () => {
  afterEach(() => vi.useRealTimers());
  test("live reactions ignore other sessions and duplicates and expire", async () => {
    vi.useFakeTimers();
    const { result } = renderHook(() => useSessionReactions("session", "fa", true));
    await act(async () => {});
    act(() => {
      result.current.handleMessage({ ...reaction("other"), session_id: "other" });
      result.current.handleMessage(reaction("one"));
      result.current.handleMessage(reaction("one"));
    });
    expect(result.current.reactions).toHaveLength(1);
    await act(async () => vi.advanceTimersByTime(3500));
    expect(result.current.reactions).toHaveLength(0);
  });
  test("reaction bursts retain at most twelve animations", async () => {
    vi.useFakeTimers();
    const { result } = renderHook(() => useSessionReactions("session", "en", true));
    await act(async () => {});
    act(() => {
      for (let i = 0; i < 20; i++) {
        result.current.handleMessage(reaction(String(i)));
      }
    });
    expect(result.current.reactions).toHaveLength(12);
    expect(result.current.reactions[0].event_id).toBe("8");
  });
  test("overlay renders isolated usernames followed by the emoji", () => {
    const { container } = render(<ReactionOverlay reactions={[reaction("one")]} locale="fa" />);
    expect(screen.getByText("سارا Ahmed").tagName).toBe("BDI");
    expect(container.querySelector("[data-session-reaction]")).toHaveAttribute("dir", "rtl");
    expect(screen.getByText("👍")).toBeInTheDocument();
  });
  test("hand icon stays outlined and exposes persistent toggle state", () => {
    const onToggle = vi.fn();
    const { container } = render(
      <TooltipProvider>
        <HandControl raised disabled={false} locale="en" onToggle={onToggle} />
      </TooltipProvider>,
    );
    expect(container.querySelector("svg")).not.toHaveClass("fill-current");
    const button = screen.getByRole("button", { name: "Lower hand" });
    expect(button).toHaveAttribute("aria-pressed", "true");
    fireEvent.click(button);
    expect(onToggle).toHaveBeenCalledOnce();
  });
  test("polls and quizzes occupy a selectable stage surface", () => {
    const select = vi.fn();
    render(
      <SessionStage
        title="Session"
        status="live"
        participants={[]}
        activeSurface="activities"
        onSelectSurface={select}
        activities={<div>Activity stage content</div>}
        collaboration={<div>Whiteboard content</div>}
        locale="en"
      />,
    );
    expect(screen.getByText("Activity stage content")).toBeInTheDocument();
    expect(screen.queryByText("Whiteboard content")).not.toBeInTheDocument();
    const button = screen.getByRole("button", { name: "Polls & quizzes" });
    expect(button).toHaveAttribute("aria-pressed", "true");
    fireEvent.click(button);
    expect(select).toHaveBeenCalledWith("activities");
  });
});
