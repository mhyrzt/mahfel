import { expect, test } from "@playwright/test";
import { login, post } from "../helpers/scenario-api";

test.use({ actionTimeout: 20_000, navigationTimeout: 30_000 });

test("admission and moderation remain authoritative across API replicas", async ({
  page,
  browser,
}, info) => {
  test.setTimeout(180_000);
  test.skip(
    !process.env.MAHFEL_E2E_PASSWORD,
    "Requires development identities",
  );
  await login(page, "misato");
  const learnerContext = await browser.newContext({
    baseURL: info.project.use.baseURL,
    extraHTTPHeaders: { "X-Acceptance-Replica": "2" },
  });
  const learner = await learnerContext.newPage();
  try {
    await login(learner, "shinji");
    const profile = await (
      await learner.request.get("/api/backend/v1/auth/me")
    ).json();
    const room = await post(page, "rooms", {
      title: `Moderation ${Date.now()}`,
      visibility: "invite_only",
    });
    await post(page, `rooms/${room.id}/members`, { user_id: profile.user_id });
    const admission = await page.request.put(
      `/api/backend/v1/rooms/${room.id}/admission`,
      { data: { policy: "lobby" } },
    );
    expect(admission.ok()).toBe(true);
    const session = await post(page, `rooms/${room.id}/sessions`, {
      title: "Moderated session",
      auto_record: false,
    });
    await post(page, `sessions/${session.id}/start`);
    await page.goto(`/session/${session.id}`);
    await learner.goto(`/session/${session.id}`);
    await expect(
      learner.getByText("Waiting for admission", { exact: true }),
    ).toBeVisible();
    expect(
      await learner.getByRole("button", { name: "Unmute microphone" }).count(),
    ).toBe(0);
    const managerProfile = await (
      await page.request.get("/api/backend/v1/auth/me")
    ).json();
    const approveSelf = page.waitForResponse(
      (response) =>
        response.url().endsWith(`/${managerProfile.user_id}/admit`) &&
        response.status() === 200,
    );
    await page
      .getByRole("button", {
        name: `Admit Participant ${managerProfile.user_id.slice(0, 4)}`,
      })
      .click();
    await approveSelf;
    await page.getByRole("button", { name: "Check admission" }).click();
    await expect(
      page.getByRole("button", { name: "Unmute microphone" }),
    ).toBeVisible();
    await page.getByRole("button", { name: "Admission", exact: true }).click();
    const approveLearner = page.waitForResponse(
      (response) =>
        response.url().endsWith(`/${profile.user_id}/admit`) &&
        response.status() === 200,
    );
    await page
      .getByRole("button", { name: /^Admit / })
      .click({ timeout: 20_000 });
    await approveLearner;
    await learner.getByRole("button", { name: "Check admission" }).click();
    await expect(
      learner.getByRole("button", { name: "Unmute microphone" }),
    ).toBeVisible();
    await learner.getByRole("button", { name: "Unmute microphone" }).click();
    await expect(
      learner.getByRole("button", { name: "Mute microphone" }),
    ).toBeVisible();
    await page
      .getByRole("button", { name: "Participants", exact: true })
      .first()
      .click();
    const actions = page
      .getByRole("button", { name: "Actions for Shinji Ikari" })
      .first();
    await page
      .getByRole("article", { name: "Shinji Ikari", exact: true })
      .hover();
    await actions.click({ timeout: 20_000 });
    await page
      .getByRole("menuitem", { name: "Mute participant", exact: true })
      .click();
    await expect(
      learner.getByRole("button", { name: "Mute microphone" }),
    ).toHaveCount(0);
    await expect(
      learner.getByRole("button", { name: "Unmute microphone" }),
    ).toHaveCount(0);
    await actions.click();
    await page
      .getByRole("menuitem", { name: "Unmute participant", exact: true })
      .click();
    await expect(
      learner.getByRole("button", { name: "Unmute microphone" }),
    ).toBeVisible();
    await actions.click();
    await page
      .getByRole("menuitem", { name: "Remove from session", exact: true })
      .click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Confirm action", exact: true })
      .click();
    await expect(
      learner.getByText("You were removed from this session.", { exact: true }),
    ).toBeVisible();
    await page.getByRole("button", { name: "Leave session" }).click();
    await post(page, `sessions/${session.id}/end`);
  } finally {
    await learnerContext.close();
  }
});
