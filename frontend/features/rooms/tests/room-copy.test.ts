import { expect, test } from "vitest";

import { roomCopy } from "../room-copy";

test("uses the requested Persian room and session terms", () => {
  const text = roomCopy("fa");
  expect(text.room).toBe("محفل");
  expect(text.rooms).toBe("محافل");
  expect(text.sessions).toBe("دیدارها");
  expect(text.newSession).toBe("دیدار جدید");
  expect(text.sessionSettings).toBe("تنظیمات دیدار");
});
