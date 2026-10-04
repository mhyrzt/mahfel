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
