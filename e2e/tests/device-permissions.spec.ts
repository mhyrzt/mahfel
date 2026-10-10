import { chromium, expect, test } from "@playwright/test";
import { login, post } from "../helpers/scenario-api";

test.use({ actionTimeout: 20_000, navigationTimeout: 30_000 });

test("device denial explains recovery and leaves session controls usable", async ({
  browserName,
}, info) => {
  test.setTimeout(120_000);
  test.skip(
    browserName !== "chromium" || !process.env.MAHFEL_E2E_PASSWORD,
    "Uses Chromium's real permission override",
  );
  const deniedBrowser = await chromium.launch({
    args: ["--use-fake-device-for-media-stream"],
  });
  const context = await deniedBrowser.newContext({
    baseURL: info.project.use.baseURL,
    permissions: ["camera"],
  });
  const page = await context.newPage();
  try {
    await login(page, "misato");
    const room = await post(page, "rooms", {
      title: `Permission denial ${Date.now()}`,
      visibility: "invite_only",
    });
    const session = await post(page, `rooms/${room.id}/sessions`, {
      title: "Device denial",
      auto_record: false,
    });
    await post(page, `sessions/${session.id}/start`);
    await page.goto(`/session/${session.id}`);
    const protocol = await context.newCDPSession(page);
    const { targetInfo } = await protocol.send("Target.getTargetInfo");
    await protocol.send("Browser.setPermission", {
      permission: { name: "microphone" },
      setting: "denied",
      origin: info.project.use.baseURL,
      browserContextId: targetInfo.browserContextId,
    });
    expect(
      await page.evaluate(
        async () =>
          (
            await navigator.permissions.query({
              name: "microphone" as PermissionName,
            })
          ).state,
      ),
    ).toBe("denied");
    await page.getByRole("button", { name: "Unmute microphone" }).click();
    await expect(
      page.getByText(
        "Device access is unavailable. Check browser permissions and try again.",
        { exact: true },
      ),
    ).toBeVisible();
    await protocol.send("Browser.setPermission", {
      permission: { name: "microphone" },
      setting: "granted",
      origin: info.project.use.baseURL,
      browserContextId: targetInfo.browserContextId,
    });
    await page.getByRole("button", { name: "Unmute microphone" }).click();
    await expect(
      page.getByRole("button", { name: "Mute microphone" }),
    ).toBeVisible({ timeout: 30_000 });
    await page.getByRole("button", { name: "Leave session" }).click();
    await post(page, `sessions/${session.id}/end`);
  } finally {
    await deniedBrowser.close();
  }
});
