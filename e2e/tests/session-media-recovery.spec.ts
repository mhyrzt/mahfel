import { expect, test } from "@playwright/test";
import { login, post } from "../helpers/scenario-api";

test.use({ actionTimeout: 20_000, navigationTimeout: 30_000 });

test("live camera crosses API replicas and survives peer rejoin", async ({
  page,
  browser,
  browserName,
}, info) => {
  test.setTimeout(360_000);
  test.skip(
    !process.env.MAHFEL_E2E_PASSWORD,
    "Requires local development identity fixtures",
  );
  await page.context().addCookies([
    {
      name: "NEXT_LOCALE",
      value: "en",
      url: info.project.use.baseURL as string,
    },
  ]);
  await login(page, "misato");
  const receiverContext = await browser.newContext({
    baseURL: info.project.use.baseURL,
    ignoreHTTPSErrors: !process.env.PLAYWRIGHT_BASE_URL,
    extraHTTPHeaders: { "X-Acceptance-Replica": "2" },
  });
  await receiverContext.addInitScript(() => {
    const peers: RTCPeerConnection[] = [];
    const NativePeer = window.RTCPeerConnection;
    window.RTCPeerConnection = class extends NativePeer {
      constructor(configuration?: RTCConfiguration) {
        super(configuration);
        peers.push(this);
      }
    };
    Object.assign(window, { acceptancePeers: peers });
  });
  const receiver = await receiverContext.newPage();
  try {
    await login(receiver, "shinji");
    const identityResponse = await receiver.request.get(
      "/api/backend/v1/auth/me",
    );
    expect(identityResponse.ok()).toBe(true);
    const identity = await identityResponse.json();
    const room = await post(page, "rooms", {
      title: `Milestone live ${Date.now()}`,
      visibility: "invite_only",
    });
    await post(page, `rooms/${room.id}/members`, {
      user_id: identity.user_id,
    });
    const session = await post(page, `rooms/${room.id}/sessions`, {
      title: "Cross-replica media",
      auto_record: false,
    });
    await post(page, `sessions/${session.id}/start`);
    await page.goto(`/session/${session.id}`);
    await receiver.goto(`/session/${session.id}`);
    await page
      .getByRole("button", { name: "Turn camera on" })
      .click({ timeout: 60_000 });
    await expect(
      page.getByRole("button", { name: "Turn camera off" }),
    ).toBeVisible({ timeout: 30_000 });
    const playingRemote = () =>
      receiver.locator("video").evaluateAll((videos) =>
        videos.some((video) => {
          const element = video as HTMLVideoElement;
          return (
            element.readyState >= 2 &&
            element.videoWidth > 0 &&
            element.currentTime > 0
          );
        }),
      );
    await expect.poll(playingRemote, { timeout: 40_000 }).toBe(true);
    await page.getByRole("button", { name: "Unmute microphone" }).click();
    await expect(
      page.getByRole("button", { name: "Mute microphone" }),
    ).toBeVisible();
    const playAudio = receiver.getByRole("button", {
      name: "Play participant audio",
    });
    if (await playAudio.isVisible()) {
      await playAudio.click();
    }
    await expect
      .poll(
        () =>
          receiver.locator("audio").evaluateAll((elements) =>
            elements.some((audio) => {
              const element = audio as HTMLAudioElement;
              return (
                element.readyState >= 2 &&
                element.currentTime > 0 &&
                !element.paused
              );
            }),
          ),
        { timeout: 30_000 },
      )
      .toBe(true);
    if (browserName === "chromium" && process.env.MAHFEL_E2E_AUDIO_FIXTURE) {
      const initialOrder = await receiver
        .locator("article[aria-label]")
        .evaluateAll((tiles) =>
          tiles.map((tile) => tile.getAttribute("aria-label")),
        );
      await receiver.getByRole("button", { name: "Unmute microphone" }).click();
      await expect
        .poll(() => receiver.locator(".speaking-avatar").count(), {
          timeout: 30_000,
        })
        .toBeGreaterThan(1);
      expect(
        await receiver
          .locator("article[aria-label]")
          .evaluateAll((tiles) =>
            tiles.map((tile) => tile.getAttribute("aria-label")),
          ),
      ).toEqual(initialOrder);
      await receiver.getByRole("button", { name: "Mute microphone" }).click();
    }
    await info.attach("decoded-media", {
      body: JSON.stringify(
        await receiver.locator("video, audio").evaluateAll((elements) =>
          elements.map((element) => {
            const media = element as HTMLVideoElement;
            return {
              kind: element.tagName,
              time: media.currentTime,
              readyState: media.readyState,
              decodedFrames: media.getVideoPlaybackQuality?.().totalVideoFrames,
            };
          }),
        ),
      ),
      contentType: "application/json",
    });
    if (browserName === "chromium") {
      await page
        .getByRole("button", { name: "Share screen", exact: true })
        .click();
      await expect
        .poll(() => receiver.locator("video").count(), { timeout: 30_000 })
        .toBeGreaterThan(1);
      await expect
        .poll(
          () =>
            receiver
              .locator("video")
              .evaluateAll(
                (videos) =>
                  videos.filter(
                    (video) =>
                      (video as HTMLVideoElement).videoWidth > 0 &&
                      (video as HTMLVideoElement).currentTime > 0,
                  ).length,
              ),
          { timeout: 30_000 },
        )
        .toBeGreaterThan(1);
      await page
        .getByRole("button", { name: "Share screen", exact: true })
        .click();
    }
    await receiver.evaluate(() => {
      const peers =
        (window as Window & { acceptancePeers?: RTCPeerConnection[] })
          .acceptancePeers ?? [];
      const peer = peers.find(
        (candidate) => candidate.connectionState === "connected",
      );
      if (!peer) throw new Error("No connected receiver transport");
      Object.defineProperty(peer, "connectionState", {
        configurable: true,
        value: "failed",
      });
      peer.dispatchEvent(new Event("connectionstatechange"));
    });
    await expect
      .poll(
        () =>
          receiver.evaluate(
            () =>
              (
                (window as Window & { acceptancePeers?: RTCPeerConnection[] })
                  .acceptancePeers ?? []
              ).filter((peer) => peer.connectionState === "connected").length,
          ),
        { timeout: 40_000 },
      )
      .toBeGreaterThan(0);
    await expect.poll(playingRemote, { timeout: 40_000 }).toBe(true);
    await receiver.reload();
    await expect.poll(playingRemote, { timeout: 40_000 }).toBe(true);
    const secondDevice = await browser.newContext({
      baseURL: info.project.use.baseURL,
      extraHTTPHeaders: { "X-Acceptance-Replica": "2" },
    });
    try {
      const additional = await secondDevice.newPage();
      await login(additional, "misato");
      await additional.goto(`/session/${session.id}`);
      await expect(
        additional.getByRole("button", { name: "Leave session" }),
      ).toBeVisible();
    } finally {
      await secondDevice.close();
    }
    await expect.poll(playingRemote, { timeout: 30_000 }).toBe(true);
    await expect(
      page.getByRole("button", { name: "Turn camera off" }),
    ).toBeVisible();
    await receiver.getByRole("button", { name: "Leave session" }).click();
    await expect(receiver).toHaveURL(new RegExp(`/room/${room.id}`), {
      timeout: 30_000,
    });
    await page.getByRole("button", { name: "Leave session" }).click();
    await post(page, `sessions/${session.id}/end`);
    const summary = await page.request.get(
      `/api/backend/v1/sessions/${session.id}/participation`,
    );
    expect(summary.ok()).toBe(true);
    await info.attach("participation", {
      body: await summary.body(),
      contentType: "application/json",
    });
  } finally {
    await receiverContext.close();
  }
});
