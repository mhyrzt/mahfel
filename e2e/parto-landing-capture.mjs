import { chromium } from "@playwright/test";
import { spawn } from "node:child_process";
import { mkdir } from "node:fs/promises";
const baseURL = "http://127.0.0.1:6107";
const server = spawn(
  "bun",
  ["node_modules/next/dist/bin/next", "dev", "-p", "6107", "-H", "127.0.0.1"],
  { cwd: "../frontend", stdio: "ignore", detached: true },
);
const stop = () => {
  if (server.pid) {
    try {
      process.kill(-server.pid, "SIGTERM");
    } catch {}
  }
};
process.on("exit", stop);
let browser;
try {
  let ready = false;
  for (let i = 0; i < 90; i++) {
    try {
      const response = await fetch(baseURL);
      if (response.ok) {
        ready = true;
        break;
      }
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  if (!ready) throw new Error("Landing server did not become ready");
  browser = await chromium.launch({
    headless: true,
    args: ["--no-sandbox", "--enable-unsafe-swiftshader"],
  });
  const context = await browser.newContext();
  const page = await context.newPage();
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await mkdir("artifacts/parto", { recursive: true });
  for (const [locale, theme, width, height] of [
    ["en", "light", 1440, 960],
    ["fa", "dark", 1440, 960],
    ["en", "light", 375, 812],
    ["fa", "dark", 375, 812],
    ["fa", "light", 320, 740],
    ["fa", "light", 768, 1024],
    ["fa", "dark", 1024, 768],
    ["fa", "light", 1280, 800],
  ]) {
    await context.addCookies([
      { name: "NEXT_LOCALE", value: locale, url: baseURL },
      { name: "mahfel_theme", value: theme, url: baseURL },
    ]);
    await page.setViewportSize({ width, height });
    await page.goto(baseURL);
    await page.locator("canvas").waitFor({ timeout: 120000 });
    await page.mouse.move(100, 150);
    await page.waitForTimeout(1200);
    await page.screenshot({
      path: `artifacts/parto/landing-${locale}-${theme}-${width}.png`,
      fullPage: true,
    });
    const contrast = await page.evaluate(() => {
      const canvas = document.createElement("canvas");
      canvas.width = canvas.height = 1;
      const context = canvas.getContext("2d");
      const luminance = (rgb) =>
        rgb
          .map((value) => {
            const channel = value / 255;
            return channel <= 0.04045
              ? channel / 12.92
              : ((channel + 0.055) / 1.055) ** 2.4;
          })
          .reduce(
            (sum, channel, i) => sum + channel * [0.2126, 0.7152, 0.0722][i],
            0,
          );
      const sample = (background, foreground) => {
        context.clearRect(0, 0, 1, 1);
        context.fillStyle = background;
        context.fillRect(0, 0, 1, 1);
        if (foreground) {
          context.fillStyle = foreground;
          context.fillRect(0, 0, 1, 1);
        }
        return Array.from(context.getImageData(0, 0, 1, 1).data).slice(0, 3);
      };
      const surface = getComputedStyle(
        document.querySelector("main"),
      ).backgroundColor;
      return Array.from(
        document.querySelectorAll(
          "#landing-content > p, #landing-content button, main ul li p, [data-parto-bubble] p, [data-parto-bubble] span",
        ),
      ).map((element) => {
        const style = getComputedStyle(element);
        const bubble = element.closest("[data-parto-bubble]");
        const background =
          element.tagName === "BUTTON"
            ? style.backgroundColor
            : bubble
              ? getComputedStyle(bubble).backgroundColor
              : surface;
        const a = luminance(sample(background));
        const b = luminance(sample(background, style.color));
        return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
      });
    });
    if (Math.min(...contrast) < 4.5)
      throw new Error(
        `Text contrast below 4.5: ${theme}/${width}/${Math.min(...contrast)}`,
      );
    if (
      await page.evaluate(
        () => document.documentElement.scrollWidth > window.innerWidth,
      )
    )
      throw new Error(`Landing overflow: ${locale}/${theme}/${width}`);
    if (
      await page
        .getByRole("button", {
          name: /Say hello again|Pause Parto|دوباره سلام کن|توقف حرکت پرتو/,
        })
        .count()
    ) {
      throw new Error("Unexpected mascot controls");
    }
  }
  await page.keyboard.press("Tab");
  await page.getByRole("link", { name: /رفتن به محتوای اصلی/ }).focus();
  await page.keyboard.press("Enter");
  await page.waitForTimeout(100);
  if (
    (await page.evaluate(() => document.activeElement?.id)) !==
    "landing-content"
  ) {
    throw new Error("Skip navigation did not focus the hero");
  }
  const controls = page.locator('button[aria-haspopup="menu"]');
  await controls.nth(0).focus();
  await page.keyboard.press("Enter");
  await page.getByRole("menuitemradio", { name: "تیره", exact: true }).click();
  await page.waitForTimeout(500);
  if ((await page.locator("html").getAttribute("data-theme")) !== "dark") {
    throw new Error("Theme did not update");
  }
  await page.reload();
  if ((await page.locator("html").getAttribute("data-theme")) !== "dark") {
    throw new Error("Theme did not persist");
  }
  await controls.nth(1).click();
  await page
    .getByRole("menuitemradio", { name: "English", exact: true })
    .click();
  await page.waitForTimeout(500);
  await page.reload();
  if (
    (await page.locator("html").getAttribute("lang")) !== "en" ||
    (await page.locator("html").getAttribute("dir")) !== "ltr"
  ) {
    throw new Error("Language and direction did not persist");
  }
  const reduced = await browser.newContext({
    reducedMotion: "reduce",
    viewport: { width: 375, height: 812 },
  });
  await reduced.addCookies([
    { name: "NEXT_LOCALE", value: "fa", url: baseURL },
    { name: "mahfel_theme", value: "dark", url: baseURL },
  ]);
  const reducedPage = await reduced.newPage();
  reducedPage.on("pageerror", (error) => errors.push(error.message));
  await reducedPage.goto(baseURL);
  await reducedPage.locator("canvas").waitFor();
  await reducedPage.waitForTimeout(2500);
  if (
    (await reducedPage
      .locator("#landing-content")
      .evaluate((element) => getComputedStyle(element).animationName)) !==
    "none"
  ) {
    throw new Error("Reduced motion did not suppress entrance");
  }
  const still = await reducedPage.locator("canvas").screenshot();
  await reducedPage.waitForTimeout(500);
  if (!still.equals(await reducedPage.locator("canvas").screenshot())) {
    throw new Error("Reduced-motion mascot kept moving");
  }
  await reducedPage.screenshot({
    path: "artifacts/parto/landing-reduced-motion.png",
    fullPage: true,
  });
  await reduced.close();
  console.log(
    JSON.stringify({
      errors,
      landing: true,
      locales: 2,
      themes: 2,
      mobileOverflow: false,
      viewportWidths: [320, 375, 768, 1024, 1280, 1440],
      normalTextContrast: "at least 4.5:1",
      keyboardSkip: true,
      themeAndLocalePersist: true,
      reducedMotionStable: true,
      noMascotControls: true,
    }),
  );
  if (errors.length) process.exitCode = 1;
} finally {
  await browser?.close();
  stop();
}
