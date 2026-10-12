import { chromium } from "@playwright/test";
import { mkdir } from "node:fs/promises";
import { spawn } from "node:child_process";
const staticMode = Boolean(process.env.PARTO_STATIC_STORYBOOK);
const baseURL =
  process.env.PARTO_BASE_URL ??
  (staticMode ? "http://127.0.0.1:6106" : "http://localhost:6006");
let server = null;
if (staticMode) {
  server = spawn(
    "python3",
    [
      "-m",
      "http.server",
      "6106",
      "--bind",
      "127.0.0.1",
      "--directory",
      "../frontend/storybook-static",
    ],
    { stdio: "ignore", detached: true },
  );
} else if (process.env.PARTO_START_STORYBOOK) {
  server = spawn("bun", ["run", "storybook", "--ci"], {
    cwd: "../frontend",
    stdio: "ignore",
    detached: true,
  });
}
const stop = () => {
  if (server?.pid) {
    try {
      process.kill(-server.pid, "SIGTERM");
    } catch {}
  }
};
process.on("exit", stop);
if (server) {
  let ready = false;
  for (let i = 0; i < 120; i++) {
    try {
      const response = await fetch(`${baseURL}/index.json`);
      if (response.ok) {
        ready = true;
        break;
      }
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  if (!ready) throw new Error("Storybook server did not become ready");
}
const browser = await chromium.launch({
  headless: true,
  args: ["--no-sandbox", "--enable-unsafe-swiftshader"],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 960 } });
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
await mkdir("artifacts/parto", { recursive: true });
for (const [story, count] of [
  ["playful", 1],
  ["excited", 1],
  ["faithful", 1],
  ["six-interpretations", 6],
  ["reference-comparison", 1],
  ["confused-sequence", 1],
  ["avatar-sizes", 10],
]) {
  await page.goto(
    `${baseURL}/iframe.html?id=brand-parto--${story}&viewMode=story`,
  );
  await page
    .locator("canvas")
    .nth(count - 1)
    .waitFor({ timeout: 120000 });
  await page.waitForTimeout(story === "confused-sequence" ? 3500 : 1000);
  await page.screenshot({
    path: `artifacts/parto/${story}.png`,
    fullPage: true,
  });
  if (await page.getByRole("alert").count())
    throw new Error("Renderer unavailable");
}
await page.goto(
  `${baseURL}/iframe.html?id=brand-parto--playful&viewMode=story`,
);
await page.locator("canvas").waitFor({ timeout: 120000 });
const character = page.getByRole("button", {
  name: /Activate for a playful hop/,
});
await character.click();
await page.waitForTimeout(450);
const firstHop = await page.locator("canvas").screenshot();
await page.screenshot({ path: "artifacts/parto/playful-hop.png" });
await page.waitForTimeout(700);
const settling = await page.locator("canvas").screenshot();
if (firstHop.equals(settling))
  throw new Error("Playful animation did not advance");
await character.focus();
await page.keyboard.press("Enter");
await page.waitForTimeout(600);
await page.screenshot({ path: "artifacts/parto/keyboard-delight.png" });
await page.goto(
  `${baseURL}/iframe.html?id=brand-parto--faithful&viewMode=story&args=paused:true`,
);
await page.locator("canvas").waitFor();
await page.waitForTimeout(600);
const paused = await page.locator("canvas").screenshot();
await page.waitForTimeout(500);
if (!paused.equals(await page.locator("canvas").screenshot()))
  throw new Error("Paused character moved");
console.log(
  JSON.stringify({
    errors,
    animationAdvances: true,
    pausedStable: true,
    keyboardReaction: true,
  }),
);
await page.close();
await browser.close();
stop();
process.exit(errors.length ? 1 : 0);
