import { expect, test } from "vitest";

import { cn } from "./utils";

test("cn merges conflicting Tailwind utility classes", () => {
  expect(cn("px-2", false && "hidden", "px-4")).toBe("px-4");
});
