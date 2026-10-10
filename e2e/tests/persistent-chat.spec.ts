import { expect, test } from "@playwright/test";
import { login, post } from "../helpers/scenario-api";

test.use({ actionTimeout: 20_000, navigationTimeout: 30_000 });

test("room chat persists and streams edits without reloading history", async ({
  page,
  browser,
}, info) => {
  test.setTimeout(120_000);
  test.skip(
    !process.env.MAHFEL_E2E_PASSWORD,
    "Requires development identities",
  );
  await login(page, "misato");
  const peerContext = await browser.newContext({
    baseURL: info.project.use.baseURL,
    extraHTTPHeaders: { "X-Acceptance-Replica": "2" },
  });
  const peer = await peerContext.newPage();
  try {
    await login(peer, "shinji");
    const profile = await (
      await peer.request.get("/api/backend/v1/auth/me")
    ).json();
    const room = await post(page, "rooms", {
      title: `Persistent chat ${Date.now()}`,
      visibility: "invite_only",
    });
    await post(page, `rooms/${room.id}/members`, { user_id: profile.user_id });
    await page.goto(`/room/${room.id}`);
    await peer.goto(`/room/${room.id}`);
    await page
      .getByRole("button", { name: "Conversation", exact: true })
      .click();
    await peer
      .getByRole("button", { name: "Conversation", exact: true })
      .click();
    const content = `Durable chat ${Date.now()}`;
    await page.getByRole("textbox", { name: "Write a message" }).fill(content);
    await page
      .locator('input[type="file"][aria-label="Attach files"]')
      .setInputFiles({
        name: "realtime-proof.png",
        mimeType: "image/png",
        buffer: Buffer.from(
          "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aWOQAAAAASUVORK5CYII=",
          "base64",
        ),
      });
    await page.getByRole("button", { name: "Send", exact: true }).click();
    await expect(peer.getByText(content, { exact: true })).toBeVisible({
      timeout: 15_000,
    });
    await expect(peer.getByTestId("message-attachments")).toBeVisible({
      timeout: 20_000,
    });
    await expect(page.getByTestId("message-attachments")).toBeVisible();
    let historyReloads = 0;
    peer.on("request", (request) => {
      if (
        request.method() === "GET" &&
        new URL(request.url()).pathname ===
          `/api/backend/v1/rooms/${room.id}/messages`
      )
        historyReloads += 1;
    });
    await page.getByRole("button", { name: "Message actions" }).click();
    await page.getByRole("menuitem", { name: "Edit", exact: true }).click();
    await page
      .getByRole("textbox", { name: "Edit", exact: true })
      .fill(`${content} edited`);
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await expect(
      peer.getByText(`${content} edited`, { exact: true }),
    ).toBeVisible();
    expect(historyReloads).toBe(0);
    await peer.reload();
    await expect(
      peer.locator(
        '#session-sidebar-actions button[aria-label="Conversation"]',
      ),
    ).toBeVisible();
    await peer
      .getByRole("button", { name: "Conversation", exact: true })
      .click();
    await expect(
      peer.getByText(`${content} edited`, { exact: true }),
    ).toBeVisible();
    await page.getByRole("button", { name: "Message actions" }).click();
    await page.getByRole("menuitem", { name: "Delete", exact: true }).click();
    await expect(
      peer.getByText(`${content} edited`, { exact: true }),
    ).toHaveCount(0);
  } finally {
    await peerContext.close();
  }
});
