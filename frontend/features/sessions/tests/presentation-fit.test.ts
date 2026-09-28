import { expect, test } from "vitest";

import { fitScale } from "../components/presentation-fit";

test("fits portrait and landscape content inside the container", () => {
  expect(fitScale(1000, 500, 500, 500)).toBeCloseTo(1);
  expect(fitScale(500, 1000, 1000, 500)).toBeCloseTo(0.5);
  expect(fitScale(400, 400, 200, 100)).toBeCloseTo(2);
  expect(fitScale(200, 100, 400, 400)).toBeCloseTo(0.25);
});

test("falls back to a unit scale for empty or invalid boxes", () => {
  expect(fitScale(0, 500, 500, 500)).toBe(1);
  expect(fitScale(1000, 0, 500, 500)).toBe(1);
  expect(fitScale(1000, 500, 0, 500)).toBe(1);
});
