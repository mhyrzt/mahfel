import { useState } from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";

import { DirectionProvider } from "../direction";
import { SegmentedControl } from "../segmented-control";
import { Slider } from "../slider";
import { Progress } from "../progress";

function Preference({ dir }: Readonly<{ dir: "ltr" | "rtl" }>) {
  const [value, setValue] = useState("first");
  return (
    <DirectionProvider dir={dir}>
      <SegmentedControl
        label="Preference"
        options={[
          { value: "first", label: "First" },
          { value: "second", label: "Second" },
        ]}
        value={value}
        onValueChange={setValue}
      />
    </DirectionProvider>
  );
}

test.each(["ltr", "rtl"] as const)(
  "exclusive preferences support arrow keys in %s",
  async (dir) => {
    render(<Preference dir={dir} />);
    const first = screen.getByRole("radio", { name: "First" });
    first.focus();
    fireEvent.keyDown(first, { key: dir === "rtl" ? "ArrowLeft" : "ArrowRight" });
    await waitFor(() => expect(screen.getByRole("radio", { name: "Second" })).toBeChecked());
    expect(first).not.toBeChecked();
  },
);

test("media seek remains LTR inside an RTL page and exposes the thumb label", () => {
  const seek = vi.fn();
  render(
    <DirectionProvider dir="rtl">
      <Slider
        aria-label="Seek recording"
        dir="ltr"
        defaultValue={[2000]}
        min={0}
        max={10000}
        step={1000}
        onValueChange={seek}
      />
    </DirectionProvider>,
  );
  fireEvent.keyDown(screen.getByRole("slider", { name: "Seek recording" }), { key: "ArrowRight" });
  expect(seek).toHaveBeenCalledWith([3000]);
});

test("export progress retains its value and maximum for assistive technology", () => {
  render(<Progress aria-label="Export" value={35} max={100} />);
  expect(screen.getByRole("progressbar", { name: "Export" })).toHaveAttribute(
    "aria-valuenow",
    "35",
  );
  expect(screen.getByRole("progressbar", { name: "Export" })).toHaveAttribute(
    "aria-valuemax",
    "100",
  );
});
