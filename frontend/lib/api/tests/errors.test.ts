import { expect, test } from "vitest";

import { apiFailureFromResponse } from "../errors";

test("preserves the API error envelope for validation feedback", () => {
  expect(
    apiFailureFromResponse(422, {
      error: { code: "unprocessable", message: "A title is required." },
    }),
  ).toEqual({
    kind: "validation",
    status: 422,
    code: "unprocessable",
    message: "A title is required.",
  });
});

test("does not expose an unrecognized failure payload", () => {
  expect(apiFailureFromResponse(503, { upstream: "database connection refused" })).toEqual({
    kind: "unavailable",
    status: 503,
    code: null,
    message: "The request could not be completed.",
  });
});
