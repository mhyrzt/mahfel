import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

const { get, post, playerProps } = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
  playerProps: { current: null as null | Record<string, unknown> },
}));

vi.mock("@/lib/api/browser", () => ({
  browserApiClient: { get, post },
}));
vi.mock("@/lib/collaboration/provider", () => ({
  MahfelCollaborationProvider: class {
    connect() {}
    destroy() {}
  },
}));
vi.mock("embla-carousel-react", () => ({
  default: () => [
    () => undefined,
    {
      selectedScrollSnap: () => 0,
      canScrollPrev: () => false,
      canScrollNext: () => false,
      scrollTo: () => undefined,
      on: () => undefined,
      off: () => undefined,
    },
  ],
}));
vi.mock("../components/presentation-player", () => ({
  PresentationPlayer: (props: Record<string, unknown>) => {
    playerProps.current = props;
    return (
      <button
        data-testid="presentation-player-stub"
        onClick={() =>
          (props.onPlayback as (state: unknown) => void)({
            positionMs: 2000,
            paused: false,
            rate: 1,
          })
        }
        type="button"
      >
        {String(props.url)}
      </button>
    );
  },
}));

import { PresentationPane } from "../components/presentation-pane";

beforeEach(() => {
  get.mockReset();
  post.mockReset();
  post.mockResolvedValue({});
  playerProps.current = null;
});

test("renders a synchronized video player and forwards playback", async () => {
  get.mockImplementation((url: string) => {
    if (url.endsWith("/collaboration/presentation")) {
      return Promise.resolve({
        data: {
          file_ids: ["file-1"],
          current_page: 1,
          total_pages: 1,
          playback: { position_ms: 1000, paused: true, rate: 1 },
        },
      });
    }
    return Promise.resolve({
      data: { url: "https://storage.test/video", headers: [], file: { mime: "video/mp4" } },
    });
  });

  render(
    <PresentationPane
      canPresent
      live
      locale="en"
      onEvent={() => undefined}
      sessionId="session-1"
    />,
  );

  const player = await screen.findByTestId("presentation-player-stub", {}, { timeout: 5000 });
  expect(player).toHaveTextContent("https://storage.test/video");
  expect(playerProps.current?.canPresent).toBe(true);
  expect(playerProps.current?.playback).toEqual({ positionMs: 1000, paused: true, rate: 1 });

  fireEvent.click(player);
  await waitFor(() =>
    expect(post).toHaveBeenCalledWith("/v1/sessions/session-1/collaboration/presentation", {
      action: "playback",
      position_ms: 2000,
      paused: false,
      rate: 1,
    }),
  );
});

test("keeps video playback read-only for viewers", async () => {
  get.mockImplementation((url: string) => {
    if (url.endsWith("/collaboration/presentation")) {
      return Promise.resolve({
        data: { file_ids: ["file-1"], current_page: 1, total_pages: 1, playback: null },
      });
    }
    return Promise.resolve({
      data: { url: "https://storage.test/video", headers: [], file: { mime: "video/webm" } },
    });
  });

  render(
    <PresentationPane
      canPresent={false}
      live
      locale="en"
      onEvent={() => undefined}
      sessionId="session-2"
    />,
  );

  await screen.findByTestId("presentation-player-stub", {}, { timeout: 5000 });
  expect(playerProps.current?.canPresent).toBe(false);
});
