import { expect, test } from "@playwright/test";
import { login, post } from "../helpers/scenario-api";

test.use({ actionTimeout: 20_000, navigationTimeout: 30_000 });

test("Persian scheduled controls support narrow screens and keyboard input", async ({
  page,
}, info) => {
  test.setTimeout(120_000);
  test.skip(
    !process.env.MAHFEL_E2E_PASSWORD,
    "Requires development identities",
  );
  await login(page, "misato");
  const room = await post(page, "rooms", {
    title: `محفل آزمون ${Date.now()}`,
    visibility: "invite_only",
  });
  await page.context().addCookies([
    {
      name: "NEXT_LOCALE",
      value: "fa",
      url: info.project.use.baseURL as string,
    },
  ]);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`/room/${room.id}`);
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  await page.getByTestId("new-session-trigger").focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("dialog")).toBeVisible();
  await page
    .getByRole("textbox", { name: "عنوان دیدار (اختیاری)" })
    .fill("دیدار آزمون");
  await page.getByLabel("مدت پیشنهادی به دقیقه (اختیاری)").fill("30");
  await page
    .getByRole("checkbox", { name: "ضبط خودکار هنگام شروع دیدار" })
    .uncheck();
  await page
    .getByTestId("create-session-form")
    .getByRole("button", { name: "دیدار جدید", exact: true })
    .click();
  await expect(page).toHaveURL(/\/session\/[0-9a-f-]+/);
  await expect(
    page.getByText(
      "این دیدار برنامه‌ریزی شده است. می‌توانید آن را زودتر هم شروع کنید.",
      { exact: true },
    ),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth + 1,
    ),
  ).toBe(true);
  await page.getByRole("button", { name: "شروع دیدار", exact: true }).focus();
  await expect(
    page.getByRole("button", { name: "شروع دیدار", exact: true }),
  ).toBeFocused();
  const sessionId = new URL(page.url()).pathname.split("/").at(-1);
  await post(page, `sessions/${sessionId}/cancel`);
});
