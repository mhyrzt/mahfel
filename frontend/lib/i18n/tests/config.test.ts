import { expect, test } from "vitest";

import { defaultLocale, directionForLocale, isLocale } from "../config";
import { messagesFor } from "../messages";

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

test("uses the Mahfel domain terms in Persian navigation and session UI", () => {
  const messages = messagesFor("fa");
  expect(messages.shell.rooms).toBe("محافل");
  expect(messages.shell.sessions).toBe("دیدارها");
  expect(messages.shell.createRoom).toBe("ایجاد محفل");
  expect(messages.sessions.sessionTitle).toBe("دیدار");
  expect(messages.sessions.blockParticipant).toBe("مسدود کردن در محفل");
});
