import { expect, test } from "@playwright/test";
import { login, post } from "../helpers/scenario-api";

test.use({ actionTimeout: 20_000, navigationTimeout: 30_000 });

test("session creation editing and terminal lifecycle use durable state", async ({
  page,
}) => {
  test.setTimeout(150_000);
  test.skip(
    !process.env.MAHFEL_E2E_PASSWORD,
    "Requires development identities",
  );
  await login(page, "misato");
  const room = await post(page, "rooms", {
    title: `Lifecycle ${Date.now()}`,
    visibility: "invite_only",
  });
  await page.goto(`/room/${room.id}`);
  await page.getByTestId("new-session-trigger").click();
  await page
    .getByRole("textbox", { name: "Session title (optional)" })
    .fill("Scheduled browser fixture");
  await page.getByLabel("Advisory duration in minutes (optional)").fill("45");
  await page
    .getByRole("checkbox", {
      name: "Record automatically when the session starts",
    })
    .uncheck();
  await page
    .getByTestId("create-session-form")
    .getByRole("button", { name: "New session", exact: true })
    .click();
  await expect(page).toHaveURL(/\/session\/[0-9a-f-]+/);
  const sessionId = new URL(page.url()).pathname.split("/").at(-1);
  await expect(
    page.getByText(
      "This session is scheduled. Its planned time does not prevent an early start.",
      { exact: true },
    ),
  ).toBeVisible();
  const invites = page.getByTestId("session-invite-manager");
  await invites
    .getByRole("button", { name: "Create link", exact: true })
    .click();
  await expect(invites.locator("code")).toHaveCount(1);
  await invites.getByRole("button", { name: /^Revoke link / }).click();
  await expect(invites.getByText("revoked", { exact: true })).toBeVisible();
  await page.goto(`/room/${room.id}`);
  await page.getByTestId(`session-settings-${sessionId}`).click();
  await page
    .getByRole("textbox", { name: "Session title (optional)" })
    .fill("");
  await page.getByLabel("Advisory duration in minutes (optional)").fill("");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  const edited = await (
    await page.request.get(`/api/backend/v1/sessions/${sessionId}`)
  ).json();
  expect(edited.title_overridden).toBe(false);
  expect(edited.scheduled_duration_minutes).toBeNull();
  await page.goto(`/session/${sessionId}`);
  await page
    .getByRole("button", { name: "Start session", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Leave session" }),
  ).toBeVisible();
  const invalid = await page.request.post(
    `/api/backend/v1/sessions/${sessionId}/start`,
  );
  expect(invalid.status()).toBe(409);
  await page
    .getByRole("button", { name: "End session for everyone", exact: true })
    .click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "End session for everyone", exact: true })
    .click();
  await page.reload();
  await expect(
    page.getByRole("region", { name: "Participation summary" }),
  ).toBeVisible();
  const cancelled = await post(page, `rooms/${room.id}/sessions`, {
    title: "Cancelled fixture",
    auto_record: false,
  });
  await page.goto(`/session/${cancelled.id}`);
  await page
    .getByRole("button", { name: "Cancel session", exact: true })
    .click();
  await expect(
    page.getByText("This session was cancelled.", { exact: true }),
  ).toBeVisible();
});
