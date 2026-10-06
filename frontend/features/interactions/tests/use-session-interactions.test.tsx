import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { browserApiClient } from "@/lib/api/browser";
import { useSessionInteractions } from "../use-session-interactions";
import { snapshot } from "../stories/fixtures";

vi.mock("@/lib/api/browser", () => ({ browserApiClient: { get: vi.fn(), post: vi.fn() } }));

describe("interaction snapshot recovery", () => {
  beforeEach(() => {
    vi.mocked(browserApiClient.get).mockResolvedValue({ data: snapshot });
    vi.mocked(browserApiClient.post).mockResolvedValue({});
  });
  afterEach(() => vi.clearAllMocks());

  test("refreshes on newer session events and ignores unrelated or duplicate versions", async () => {
    const { result } = renderHook(() => useSessionInteractions("session", "en", true));
    await waitFor(() => expect(result.current.snapshot?.version).toBe(4));
    act(() => {
      result.current.handleMessage({
        v: 1,
        type: "session_interactions_changed",
        session_id: "other",
        version: 5,
      });
      result.current.handleMessage({
        v: 1,
        type: "session_interactions_changed",
        session_id: "session",
        version: 4,
      });
    });
    expect(browserApiClient.get).toHaveBeenCalledTimes(1);
    vi.mocked(browserApiClient.get).mockResolvedValueOnce({
      data: { ...snapshot, version: 5, activities: [] },
    });
    act(() =>
      result.current.handleMessage({
        v: 1,
        type: "session_interactions_changed",
        session_id: "session",
        version: 5,
      }),
    );
    await waitFor(() => expect(result.current.snapshot?.version).toBe(5));
    expect(browserApiClient.get).toHaveBeenCalledTimes(2);
  });

  test("reuses the request identifier after an uncertain network failure", async () => {
    const { result } = renderHook(() => useSessionInteractions("session", "en", true));
    await waitFor(() => expect(result.current.snapshot).not.toBeNull());
    vi.mocked(browserApiClient.post).mockRejectedValueOnce(new Error("connection lost"));
    const command = { action: "set_hand", raised: true, expected_version: 0 } as const;
    await act(async () => {
      expect(await result.current.send(command)).toBe(false);
    });
    await act(async () => {
      expect(await result.current.send(command)).toBe(true);
    });
    const calls = vi.mocked(browserApiClient.post).mock.calls;
    expect(calls[0][1]).toEqual(calls[1][1]);
    expect(calls[0][1]).toEqual(expect.objectContaining({ request_id: expect.any(String) }));
  });

  test("refuses mutations while not joined", async () => {
    const { result } = renderHook(() => useSessionInteractions("session", "en", false));
    await act(async () => {
      expect(
        await result.current.send({ action: "set_hand", raised: true, expected_version: 0 }),
      ).toBe(false);
    });
    expect(browserApiClient.post).not.toHaveBeenCalled();
    expect(browserApiClient.get).not.toHaveBeenCalled();
  });
});
