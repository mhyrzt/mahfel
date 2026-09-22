import { getToken } from "next-auth/jwt";
import { NextResponse, type NextRequest } from "next/server";
import { sessionCookieName, shouldUseSecureAuthCookies } from "./lib/auth/cookie-policy";

const PUBLIC_PATHS = ["/", "/login", "/api/auth", "/api/health"];

function isPublicPath(pathname: string): boolean {
  return PUBLIC_PATHS.some((path) => pathname === path || pathname.startsWith(`${path}/`));
}

export async function middleware(request: NextRequest): Promise<NextResponse> {
  const { pathname, search } = request.nextUrl;
  const publicPath = isPublicPath(pathname);
  const token = await getToken({
    req: request,
    secret: process.env.NEXTAUTH_SECRET,
    secureCookie: shouldUseSecureAuthCookies(),
    cookieName: sessionCookieName(),
  });

  if (pathname !== "/" && token?.error && token.error !== "RefreshTemporarilyUnavailable") {
    if (pathname.startsWith("/api/")) {
      return NextResponse.json({ error: "SessionExpired" }, { status: 401 });
    }
    const loginUrl = new URL("/login", request.url);
    if (!publicPath) {
      loginUrl.searchParams.set("callbackUrl", `${pathname}${search}`);
    }
    if (!loginUrl.searchParams.has("error")) {
      loginUrl.searchParams.set("error", "SessionExpired");
    }
    return NextResponse.redirect(loginUrl);
  }

  if (!publicPath && !token) {
    if (pathname.startsWith("/api/")) {
      return NextResponse.json({ error: "AuthenticationRequired" }, { status: 401 });
    }
    const loginUrl = new URL("/login", request.url);
    loginUrl.searchParams.set("callbackUrl", `${pathname}${search}`);
    return NextResponse.redirect(loginUrl);
  }

  return NextResponse.next();
}

export const config = {
  matcher: ["/((?!_next/static|_next/image|favicon.ico).*)"],
};
