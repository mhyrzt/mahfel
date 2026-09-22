import { expect, test } from "vitest";

import { browserApiClient } from "../browser";

test("uses the authenticated same-origin BFF", () => {
  expect(browserApiClient.defaults.baseURL).toBe("/api/backend");
  expect(browserApiClient.defaults.timeout).toBe(10_000);
});
