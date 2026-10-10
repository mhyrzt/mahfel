import { NextRequest } from "next/server";
import { expect, test, vi } from "vitest";
vi.mock("next-auth/jwt", () => ({ getToken: vi.fn().mockResolvedValue(null) }));
vi.mock("../config", () => ({
  AUTH_CONFIG: {
    nextAuthUrl: "https://web.example",
    nextAuthSecret: "fixture",
    keycloak: { issuer: "https://auth.example/realms/test", clientId: "web" },
  },
}));
import { keycloakLogout } from "../keycloak";

test.each([
  "//evil.example",
  "/\\evil.example",
  "https://evil.example",
  "//web.example@evil.example",
])("keeps logout callback %s on the configured origin", async (callback) => {
  const response = await keycloakLogout(
    new NextRequest(
      `https://web.example/api/auth/federated-logout?callbackUrl=${encodeURIComponent(callback)}`,
    ),
  );
  const { url } = await response.json();
  const redirect = new URL(
    new URL(url).searchParams.get("post_logout_redirect_uri") ?? "https://invalid.example",
  );
  expect(redirect.origin).toBe("https://web.example");
  expect(redirect.pathname).toBe("/login");
});
test("preserves an ordinary same-origin relative callback", async () => {
  const response = await keycloakLogout(
    new NextRequest(
      "https://web.example/api/auth/federated-logout?callbackUrl=%2Frooms%3Fpage%3D2",
    ),
  );
  const { url } = await response.json();
  expect(new URL(url).searchParams.get("post_logout_redirect_uri")).toBe(
    "https://web.example/rooms?page=2",
  );
});
