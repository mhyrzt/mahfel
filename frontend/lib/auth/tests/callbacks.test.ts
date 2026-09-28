import { afterEach, beforeEach, expect, test, vi } from "vitest";
import type { JWT } from "next-auth/jwt";

const refreshUrl = "http://keycloak:8080/realms/rastai/protocol/openid-connect/token";

function storedToken(expiresAt: number): JWT {
  return {
    accessToken: "old-access",
    refreshToken: "old-refresh",
    idToken: "id-token",
    expiresAt,
    refreshExpiresAt: Date.now() + 30 * 60_000,
  };
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date("2026-09-26T12:00:00Z"));
  vi.stubEnv("KEYCLOAK_TOKEN_ENDPOINT", refreshUrl);
  vi.resetModules();
});

afterEach(() => {
  vi.unstubAllEnvs();
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

test("refreshes before access expiry through the server-only Keycloak endpoint", async () => {
  const fetch = vi.fn().mockResolvedValue(
    Response.json({
      access_token: "new-access",
      refresh_token: "new-refresh",
      expires_in: 300,
      refresh_expires_in: 1800,
    }),
  );
  vi.stubGlobal("fetch", fetch);
  const { jwtCallback } = await import("../callbacks");

  const refreshed = await jwtCallback({ token: storedToken(Date.now() + 90_000) });

  expect(fetch).toHaveBeenCalledWith(refreshUrl, expect.objectContaining({ method: "POST" }));
  expect(refreshed).toMatchObject({
    accessToken: "new-access",
    refreshToken: "new-refresh",
    expiresAt: Date.now() + 300_000,
    refreshExpiresAt: Date.now() + 1_800_000,
  });
  expect(refreshed.error).toBeUndefined();
});

test("keeps a valid access token until its refresh window", async () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  const { jwtCallback } = await import("../callbacks");
  const token = storedToken(Date.now() + 180_000);

  expect(await jwtCallback({ token })).toEqual(token);
  expect(fetch).not.toHaveBeenCalled();
});

test("distinguishes a rejected refresh token from a temporary outage", async () => {
  const { jwtCallback } = await import("../callbacks");
  const expired = storedToken(Date.now() - 1);
  vi.stubGlobal(
    "fetch",
    vi.fn().mockResolvedValue(Response.json({ error: "invalid_grant" }, { status: 400 })),
  );

  expect(await jwtCallback({ token: expired })).toMatchObject({
    error: "RefreshAccessTokenError",
  });

  vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("network unavailable")));
  expect(await jwtCallback({ token: expired })).toMatchObject({
    error: "RefreshTemporarilyUnavailable",
    refreshRetryAt: Date.now() + 5_000,
  });
});

test("uses a valid access token during a temporary refresh outage only until expiry", async () => {
  const { accessTokenFromServerSession } = await import("../callbacks");
  const session = {
    accessToken: "still-valid",
    accessTokenExpiresAt: Date.now() + 60_000,
    error: "RefreshTemporarilyUnavailable",
  };

  expect(accessTokenFromServerSession(session)).toBe("still-valid");
  expect(accessTokenFromServerSession(session, Date.now() + 60_000)).toBeNull();
  expect(accessTokenFromServerSession({ ...session, error: "RefreshAccessTokenError" })).toBeNull();
});
