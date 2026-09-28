import { expect, test } from "vitest";

import { shouldCorrectDrift } from "../components/presentation-playback";

test("only corrects playback when the remote position drifts past the threshold", () => {
  expect(shouldCorrectDrift(10_000, 10_200)).toBe(false);
  expect(shouldCorrectDrift(10_000, 10_500)).toBe(false);
  expect(shouldCorrectDrift(10_000, 12_000)).toBe(true);
  expect(shouldCorrectDrift(12_000, 10_000)).toBe(true);
  expect(shouldCorrectDrift(0, 900, 500)).toBe(true);
});
