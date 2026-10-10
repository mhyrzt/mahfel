import { getToken } from "next-auth/jwt";
import { NextResponse, type NextRequest } from "next/server";
import { AUTH_CONFIG } from "./config";
import { sessionCookieName, shouldUseSecureAuthCookies } from "./cookie-policy";

export async function keycloakLogout(request: NextRequest): Promise<NextResponse> {
  const useSecureCookies = shouldUseSecureAuthCookies();
  const token = await getToken({
    req: request,
    secret: AUTH_CONFIG.nextAuthSecret,
    secureCookie: useSecureCookies,
    cookieName: sessionCookieName(),
  });
  const logoutUrl = new URL(`${AUTH_CONFIG.keycloak.issuer}/protocol/openid-connect/logout`);
  logoutUrl.searchParams.set("client_id", AUTH_CONFIG.keycloak.clientId);
  const callbackUrl = request.nextUrl.searchParams.get("callbackUrl");
  const base = new URL(AUTH_CONFIG.nextAuthUrl);
  let destination = new URL("/login?loggedOut=1", base);
  if (
    callbackUrl?.startsWith("/") &&
    !callbackUrl.startsWith("//") &&
    !callbackUrl.includes("\\")
  ) {
    const candidate = new URL(callbackUrl, base);
    if (candidate.origin === base.origin) {
      destination = candidate;
    }
  }
  const postLogoutRedirectUri = destination.toString();
  logoutUrl.searchParams.set("post_logout_redirect_uri", postLogoutRedirectUri);

  if (typeof token?.idToken === "string") {
    logoutUrl.searchParams.set("id_token_hint", token.idToken);
  }

  return NextResponse.json({ url: logoutUrl.toString() });
}
