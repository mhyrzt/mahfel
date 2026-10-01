import { chromium, expect, test as base, type Page } from "@playwright/test";
import { writeFile } from "node:fs/promises";

const test = base.extend<{ recorder: Page }>({
  recorder: async ({ page, browserName }, provideRecorder) => {
    if (browserName === "chromium") {
      await provideRecorder(page);
      return;
    }
    const browser = await chromium.launch({
      args: [
        "--use-fake-device-for-media-stream",
        "--use-fake-ui-for-media-stream",
        "--auto-select-desktop-capture-source=Entire screen",
      ],
    });
    const context = await browser.newContext({
      baseURL: process.env.PLAYWRIGHT_BASE_URL ?? "https://mahfel.localhost",
      ignoreHTTPSErrors: !process.env.PLAYWRIGHT_BASE_URL,
      permissions: ["microphone", "camera"],
    });
    try {
      await provideRecorder(await context.newPage());
    } finally {
      await browser.close();
    }
  },
});

const password = process.env.MAHFEL_E2E_PASSWORD;

async function login(
  page: Page,
  username = process.env.MAHFEL_E2E_USERNAME ?? "misato",
) {
  await page.goto("/login?callbackUrl=%2Fworkspace");
  await page.getByRole("button", { name: "Sign in with Keycloak" }).click();
  await page.locator("#username").fill(username);
  await page.locator("#password").fill(password ?? "");
  await page.locator("#kc-login").click();
  await expect(page).toHaveURL(/\/workspace/, { timeout: 30_000 });
}

test("a recorded session replays independent microphone and camera tracks", async ({
  page,
  recorder,
  browser,
  browserName,
}, testInfo) => {
  test.setTimeout(240_000);
  test.skip(
    !password,
    "Set MAHFEL_E2E_PASSWORD for the local development account",
  );

  await login(recorder);

  const post = async (path: string, data?: unknown) => {
    const response = await recorder.request.post(`/api/backend/v1/${path}`, {
      data,
    });
    expect(response.ok(), `${path}: ${response.status()}`).toBe(true);
    return response.json() as Promise<Record<string, unknown>>;
  };
  const room = await post("rooms", {
    title: `M9 browser replay ${Date.now()}`,
    visibility: "invite_only",
  });
  const roomId = String(room.id);
  const session = await post(`rooms/${roomId}/sessions`, {
    title: "Recorded browser fixture",
    auto_record: true,
  });
  const sessionId = String(session.id);
  const started = await post(`sessions/${sessionId}/start`);
  expect(started.status).toBe("live");

  await recorder.goto(`/session/${sessionId}`);
  await recorder
    .getByRole("button", { name: "Unmute microphone" })
    .click({ timeout: 30_000 });
  await recorder.getByRole("button", { name: "Turn camera on" }).click();
  await expect(
    recorder.getByRole("button", { name: "Mute microphone" }),
  ).toBeVisible();
  await expect(
    recorder.getByRole("button", { name: "Turn camera off" }),
  ).toBeVisible();
  await recorder
    .getByRole("button", { name: "Share screen", exact: true })
    .click();
  await recorder.getByRole("button", { name: "Notes", exact: true }).click();
  const notes = recorder.getByRole("textbox", { name: "Collaborative notes" });
  await notes.fill("M9 synchronized notes");
  await recorder.waitForTimeout(3_000);
  await recorder
    .getByRole("button", { name: "Share screen", exact: true })
    .click();
  await recorder.waitForTimeout(7_000);
  await post(`sessions/${sessionId}/end`);

  let recordingId = "";
  await expect
    .poll(
      async () => {
        const response = await recorder.request.get(
          `/api/backend/v1/sessions/${sessionId}/recordings`,
        );
        if (!response.ok()) {
          return "unavailable";
        }
        const recordings = (await response.json()) as Array<{
          id: string;
          status: string;
        }>;
        recordingId = recordings[0]?.id ?? "";
        return recordings[0]?.status ?? "missing";
      },
      { timeout: 150_000, intervals: [1_000, 2_000, 5_000] },
    )
    .toBe("ready");

  const descriptor = await recorder.request.get(
    `/api/backend/v1/sessions/${sessionId}/recordings/${recordingId}/playback`,
  );
  expect(descriptor.ok()).toBe(true);
  const replay = (await descriptor.json()) as {
    media: Array<{ audio: boolean; source: string }>;
  };
  expect(replay.media.some((segment) => segment.audio)).toBe(true);
  expect(replay.media.some((segment) => !segment.audio)).toBe(true);
  expect(replay.media.some((segment) => segment.source === "screen")).toBe(
    true,
  );

  if (page !== recorder) {
    await login(page);
  }

  await page.goto(`/room/${roomId}`);
  const card = page.getByTestId(`room-session-${sessionId}`);
  await card.getByRole("button", { name: "Recordings" }).click();
  await page.getByRole("button", { name: "Play recording" }).click();
  const player = page.getByRole("dialog", { name: "Recording playback" });
  await expect(
    player.getByRole("button", { name: "Play", exact: true }),
  ).toBeEnabled();
  await player.getByRole("button", { name: "Play", exact: true }).click();
  await expect
    .poll(async () => player.locator("video").count(), { timeout: 20_000 })
    .toBeGreaterThan(1);
  await expect
    .poll(
      async () =>
        player
          .locator("video")
          .first()
          .evaluate((video: HTMLVideoElement) => video.currentTime),
      { timeout: 20_000 },
    )
    .toBeGreaterThan(0.5);
  await expect
    .poll(
      async () =>
        player
          .locator("audio")
          .first()
          .evaluate((audio: HTMLAudioElement) => audio.currentTime),
      { timeout: 20_000 },
    )
    .toBeGreaterThan(0.5);
  await player.getByRole("button", { name: "Pause", exact: true }).click();
  await expect
    .poll(async () =>
      player
        .locator("audio")
        .first()
        .evaluate((audio: HTMLAudioElement) => audio.paused),
    )
    .toBe(true);
  await player.getByRole("checkbox").first().uncheck();
  await expect
    .poll(async () =>
      player
        .locator("audio")
        .first()
        .evaluate((audio: HTMLAudioElement) => audio.muted),
    )
    .toBe(true);
  const position = player.getByRole("slider", { name: "Playback position" });
  await position.press("End");
  await position.press("ArrowLeft");
  await expect(
    player.getByText("M9 synchronized notes", { exact: false }),
  ).toBeVisible();
  await testInfo.attach("recording-replay", {
    body: await player.screenshot(),
    contentType: "image/png",
  });

  const outsiderContext = await browser.newContext({
    baseURL: process.env.PLAYWRIGHT_BASE_URL ?? "https://mahfel.localhost",
    ignoreHTTPSErrors: !process.env.PLAYWRIGHT_BASE_URL,
  });
  try {
    const outsider = await outsiderContext.newPage();
    await login(outsider, "shinji");
    for (const suffix of [
      "playback",
      "playback/media-window?start_ms=0",
      "export/download",
    ]) {
      const response = await outsider.request.get(
        `/api/backend/v1/sessions/${sessionId}/recordings/${recordingId}/${suffix}`,
      );
      expect(response.status()).toBe(404);
    }
  } finally {
    await outsiderContext.close();
  }

  if (browserName === "chromium") {
    const exportPath = `/api/backend/v1/sessions/${sessionId}/recordings/${recordingId}/export`;
    let size = 0;
    await expect
      .poll(
        async () => {
          const response = await recorder.request.get(exportPath);
          expect(response.ok()).toBe(true);
          const result = (await response.json()) as {
            status: string;
            size_bytes: number;
          };
          size = result.size_bytes;
          return result.status;
        },
        { timeout: 150_000, intervals: [1_000, 2_000, 5_000] },
      )
      .toBe("ready");
    const download = await recorder.request.get(`${exportPath}/download`);
    expect(download.ok()).toBe(true);
    const signed = (await download.json()) as { url: string };
    const artifact = await recorder.request.get(signed.url);
    expect(artifact.ok()).toBe(true);
    const bytes = await artifact.body();
    expect(bytes.length).toBe(size);
    expect(bytes.subarray(4, 8).toString()).toBe("ftyp");
    const path = testInfo.outputPath("recording-export.mp4");
    await writeFile(path, bytes);
    await testInfo.attach("recording-export", {
      path,
      contentType: "video/mp4",
    });
  }
});
