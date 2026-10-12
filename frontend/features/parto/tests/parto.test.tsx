import { act, fireEvent, render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Parto } from "../components/parto";
const mocks = vi.hoisted(() => ({
  update: vi.fn(),
  destroy: vi.fn(),
  play: vi.fn(),
  restart: vi.fn(),
  create: vi.fn(),
}));
vi.mock("../core/renderer", () => ({ createParto: mocks.create }));
beforeEach(() => {
  vi.clearAllMocks();
  mocks.create.mockResolvedValue(mocks);
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: () => ({ matches: false, addEventListener: vi.fn(), removeEventListener: vi.fn() }),
  });
});
describe("Parto lifecycle", () => {
  it("updates inputs without recreating the renderer and cleans up", async () => {
    const view = render(<Parto activity="idle" />);
    await waitFor(() => expect(mocks.create).toHaveBeenCalledTimes(1));
    view.rerender(<Parto activity="speaking" audioLevel={0.7} />);
    expect(mocks.update).toHaveBeenCalledWith(
      expect.objectContaining({ activity: "speaking", audioLevel: 0.7 }),
    );
    expect(mocks.create).toHaveBeenCalledTimes(1);
    view.unmount();
    expect(mocks.destroy).toHaveBeenCalledTimes(1);
  });
  it("destroys a renderer that resolves after unmount", async () => {
    let resolve: (value: typeof mocks) => void = () => undefined;
    mocks.create.mockReturnValue(
      new Promise<typeof mocks>((done) => {
        resolve = done;
      }),
    );
    const view = render(<Parto />);
    await waitFor(() => expect(mocks.create).toHaveBeenCalledTimes(1));
    view.unmount();
    await act(async () => resolve(mocks));
    expect(mocks.destroy).toHaveBeenCalledTimes(1);
  });
  it("reports initialization failure", async () => {
    mocks.create.mockRejectedValue(new Error("GPU unavailable"));
    const onError = vi.fn();
    const view = render(<Parto onError={onError} />);
    expect(await view.findByRole("alert")).toHaveTextContent("renderer unavailable");
    expect(onError).toHaveBeenCalled();
  });
});

it("supports keyboard delight without remounting the renderer", async () => {
  const view = render(<Parto interactive />);
  await waitFor(() => expect(mocks.create).toHaveBeenCalledTimes(1));
  fireEvent.keyDown(view.getByRole("button"), { key: "Enter" });
  expect(mocks.play).toHaveBeenCalledWith("delight");
  expect(mocks.create).toHaveBeenCalledTimes(1);
});

it("tracks a cursor outside the mascot and removes its listener on unmount", async () => {
  const bounds = vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
    x: 0,
    y: 0,
    left: 0,
    top: 0,
    right: 320,
    bottom: 320,
    width: 320,
    height: 320,
    toJSON: () => ({}),
  });
  const view = render(<Parto followCursor />);
  await waitFor(() => expect(mocks.create).toHaveBeenCalledTimes(1));
  fireEvent(window, new MouseEvent("pointermove", { clientX: 500, clientY: 160 }));
  expect(mocks.update).toHaveBeenLastCalledWith(
    expect.objectContaining({ gazeX: expect.any(Number), gazeY: 0 }),
  );
  view.unmount();
  mocks.update.mockClear();
  fireEvent(window, new MouseEvent("pointermove", { clientX: 10, clientY: 10 }));
  expect(mocks.update).not.toHaveBeenCalled();
  bounds.mockRestore();
});
