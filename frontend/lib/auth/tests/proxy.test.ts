import { getToken } from "next-auth/jwt";
import { NextRequest } from "next/server";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { proxy } from "@/proxy";

vi.mock("next-auth/jwt", () => ({ getToken: vi.fn() }));

describe("public brand asset authentication", () => {
  beforeEach(() => {
    vi.mocked(getToken).mockResolvedValue(null);
  });

  it("serves the logo to an anonymous visitor", async () => {
    const response = await proxy(new NextRequest("https://mahfel.example/mahfel-mark.svg"));

    expect(response.headers.get("x-middleware-next")).toBe("1");
    expect(response.headers.get("location")).toBeNull();
  });

  it.each(["/rooms", "/mahfel-mark.svg/private"])("requires login for %s", async (pathname) => {
    const response = await proxy(new NextRequest(`https://mahfel.example${pathname}`));
    const location = new URL(response.headers.get("location") ?? "https://invalid.example");

    expect(response.status).toBe(307);
    expect(location.pathname).toBe("/login");
    expect(location.searchParams.get("callbackUrl")).toBe(pathname);
  });
});

describe("session recovery", () => {
  it("allows the server session to retry temporary provisioning failures", async () => {
    vi.mocked(getToken).mockResolvedValue({
      accessToken: "private",
      refreshToken: "private",
      expiresAt: Date.now() + 60_000,
      refreshExpiresAt: Date.now() + 300_000,
      error: "AccountProvisioningUnavailable",
    });
    const response = await proxy(new NextRequest("https://mahfel.example/workspace"));
    expect(response.headers.get("x-middleware-next")).toBe("1");
    expect(response.headers.get("location")).toBeNull();
  });

  it("still redirects invalid refresh tokens to sign-in", async () => {
    vi.mocked(getToken).mockResolvedValue({
      accessToken: "private",
      refreshToken: "private",
      expiresAt: 0,
      refreshExpiresAt: 0,
      error: "RefreshAccessTokenError",
    });
    const response = await proxy(new NextRequest("https://mahfel.example/workspace"));
    const location = new URL(response.headers.get("location") ?? "https://invalid.example");
    expect(location.pathname).toBe("/login");
    expect(location.searchParams.get("error")).toBe("SessionExpired");
  });
});
