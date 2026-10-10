import { expect, test } from "@playwright/test";
import { login, post } from "../helpers/scenario-api";

test.use({ actionTimeout: 20_000, navigationTimeout: 30_000 });

test("invite URLs preserve confirmation and expose unusable codes deliberately", async ({
  page,
  browser,
}, info) => {
  test.setTimeout(120_000);
  test.skip(
    !process.env.MAHFEL_E2E_PASSWORD,
    "Requires development identities",
  );
  await login(page, "misato");
  const room = await post(page, "rooms", {
    title: `Invite management ${Date.now()}`,
    visibility: "invite_only",
  });
  await page.goto(`/room/${room.id}`);
  await page.getByTestId("room-settings-trigger").click();
  await page.getByRole("tab", { name: "Invite links", exact: true }).click();
  const manager = page.getByTestId("room-invite-manager");
  await manager
    .getByRole("button", { name: "Create link", exact: true })
    .click();
  const code = manager.locator("code").first();
  await expect(code).not.toBeEmpty();
  const original = await code.innerText();
  await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);
  await manager
    .getByRole("button", { name: `Copy link ${original}`, exact: true })
    .click();
  await expect(
    manager.getByText("Invite link copied.", { exact: true }),
  ).toBeVisible();
  const holderContext = await browser.newContext({
    baseURL: info.project.use.baseURL,
  });
  const holder = await holderContext.newPage();
  try {
    await login(holder, "asuka");
    await holder.goto(`/join?code=${encodeURIComponent(original)}`);
    await expect(
      holder.getByRole("textbox", { name: "Invite code" }),
    ).toHaveValue(original);
    await holder.getByRole("button", { name: "Continue", exact: true }).click();
    await expect(
      holder.getByText("Waiting for approval", { exact: true }),
    ).toBeVisible();
    expect(
      await holder.getByRole("link", { name: "Open destination" }).count(),
    ).toBe(0);
    await page.getByRole("tab", { name: "Room requests", exact: true }).click();
    await page
      .getByRole("button", { name: /^Admit / })
      .click({ timeout: 20_000 });
    await expect(
      holder.getByText("Access approved", { exact: true }),
    ).toBeVisible({ timeout: 15_000 });
    await holder.getByRole("link", { name: "Open destination" }).click();
    await expect(holder).toHaveURL(new RegExp(`/room/${room.id}`));
    expect(await holder.getByTestId("room-settings-trigger").count()).toBe(0);
    await page.getByRole("tab", { name: "Invite links", exact: true }).click();
    await manager
      .getByRole("button", { name: `Rotate code ${original}`, exact: true })
      .click();
    await expect(
      manager.locator("code").filter({ hasText: original }),
    ).toHaveCount(1);
    await expect(manager.locator("code")).toHaveCount(2);
    const active = manager
      .locator("li")
      .filter({ has: page.getByRole("button", { name: /^Revoke link / }) });
    const fresh = await active.locator("code").innerText();
    expect(fresh).not.toBe(original);
    await manager
      .getByRole("button", { name: `Revoke link ${fresh}`, exact: true })
      .click();
    await expect(
      manager.getByRole("button", { name: /^Revoke link / }),
    ).toHaveCount(0);
    await holder.goto(`/join?code=${encodeURIComponent(fresh)}`);
    await holder.getByRole("button", { name: "Continue", exact: true }).click();
    await expect(
      holder.getByText("Invite unavailable", { exact: true }),
    ).toBeVisible();
  } finally {
    await holderContext.close();
  }
});
