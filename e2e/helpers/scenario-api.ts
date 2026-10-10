import { expect, type Page } from "@playwright/test";

export async function login(page: Page, username: string) {
  await page.goto("/login?callbackUrl=%2Fworkspace");
  await page.getByRole("button", { name: "Sign in with Keycloak" }).click();
  await page.locator("#username").fill(username);
  await page.locator("#password").fill(process.env.MAHFEL_E2E_PASSWORD ?? "");
  await page.locator("#kc-login").click();
  await expect(page).toHaveURL(/\/workspace/, { timeout: 60_000 });
}

export async function post(page: Page, path: string, data?: unknown) {
  const response = await page.request.post(`/api/backend/v1/${path}`, { data });
  expect(response.ok(), `${path}: ${response.status()}`).toBe(true);
  return response.json();
}
