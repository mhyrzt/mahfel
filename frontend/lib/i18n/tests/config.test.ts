import { expect, test } from "vitest";

import { defaultLocale, directionForLocale, isLocale } from "../config";

test("recognizes the supported English and Persian locales", () => {
  expect(isLocale("en")).toBe(true);
  expect(isLocale("fa")).toBe(true);
  expect(isLocale("de")).toBe(false);
  expect(defaultLocale).toBe("en");
});

test("uses RTL only for Persian", () => {
  expect(directionForLocale("en")).toBe("ltr");
  expect(directionForLocale("fa")).toBe("rtl");
});
