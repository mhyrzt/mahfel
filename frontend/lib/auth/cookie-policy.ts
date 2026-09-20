import type { NextRequest, NextResponse } from "next/server";

const INSECURE_COOKIE_NAMES = [
  "next-auth.session-token",
  "next-auth.callback-url",
  "next-auth.csrf-token",
  "next-auth.pkce.code_verifier",
  "next-auth.state",
  "next-auth.nonce",
] as const;

const SECURE_COOKIE_NAMES = [
  "__Secure-next-auth.session-token",
  "__Secure-next-auth.callback-url",
  "__Host-next-auth.csrf-token",
  "__Secure-next-auth.pkce.code_verifier",
  "__Secure-next-auth.state",
  "__Secure-next-auth.nonce",
] as const;

export const AUTH_COOKIE_NAMES = [...INSECURE_COOKIE_NAMES, ...SECURE_COOKIE_NAMES] as const;

export function shouldUseSecureAuthCookies(env: NodeJS.ProcessEnv = process.env): boolean {
  const configured = env.APP_COOKIE_SECURE?.trim().toLowerCase();
  const production = env.NODE_ENV === "production";
  if (production && configured === "false") {
    throw new Error("APP_COOKIE_SECURE=false is not allowed when NODE_ENV=production");
  }
  if (production) {
    return true;
  }
  return configured === "true";
}

export function sessionCookieName(env: NodeJS.ProcessEnv = process.env): string {
  return `${shouldUseSecureAuthCookies(env) ? "__Secure-" : ""}next-auth.session-token`;
}

export function hasAuthSessionCookie(request: NextRequest): boolean {
  return request.cookies
    .getAll()
    .some(({ name }) => /^(?:__Secure-)?next-auth\.session-token(?:\.\d+)?$/.test(name));
}

export function clearAuthCookies(
  request: NextRequest,
  response: NextResponse,
  scope: "session" | "all" = "all",
): void {
  const baseNames =
    scope === "session"
      ? AUTH_COOKIE_NAMES.filter((name) => name.endsWith("session-token"))
      : AUTH_COOKIE_NAMES;
  const names = new Set<string>(baseNames);
  for (const cookie of request.cookies.getAll()) {
    if (baseNames.some((name) => cookie.name === name || cookie.name.startsWith(`${name}.`))) {
      names.add(cookie.name);
    }
  }
  for (const name of names) {
    response.cookies.set({
      name,
      value: "",
      expires: new Date(0),
      maxAge: 0,
      path: "/",
      httpOnly: true,
      sameSite: "lax",
      secure: name.startsWith("__Secure-") || name.startsWith("__Host-"),
    });
  }
}
