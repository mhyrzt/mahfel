import type { JWT } from "next-auth/jwt";
import type { Session } from "next-auth";
import { AUTH_CONFIG } from "./config";
import { extractUserFromToken } from "./jwt";
import { serverApiBaseUrl } from "../api/config";

interface ExtendedJWT extends JWT {
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
  refreshExpiresAt: number;
  idToken?: string;
  userId?: string;
  avatarUrl?: string;
  roles?: string[];
  error?: string;
  refreshRetryAt?: number;
}

const AUTH_UPSTREAM_TIMEOUT_MS = 10_000;
const REFRESH_RETRY_MS = 5_000;
const ACCESS_REFRESH_BUFFER_MS = 120_000;

interface SignInAccount {
  access_token?: unknown;
  app_user_id?: string;
}

export const signInCallback = async ({
  account,
}: {
  account?: SignInAccount | null;
}): Promise<boolean> => {
  if (!account || typeof account.access_token !== "string") {
    return false;
  }
  return true;
};

async function exchangeKeycloakToken(token: ExtendedJWT): Promise<ExtendedJWT> {
  try {
    const params = new URLSearchParams();
    params.append("grant_type", "refresh_token");
    params.append("client_id", AUTH_CONFIG.keycloak.clientId);
    params.append("client_secret", AUTH_CONFIG.keycloak.clientSecret);
    params.append("refresh_token", token.refreshToken);
    const response = await fetch(AUTH_CONFIG.keycloak.tokenEndpoint, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: params.toString(),
      signal: AbortSignal.timeout(AUTH_UPSTREAM_TIMEOUT_MS),
    });
    if (!response.ok) {
      const body = (await response.json().catch(() => null)) as { error?: string } | null;
      const isInvalidGrant = response.status === 400 && body?.error === "invalid_grant";
      return {
        ...token,
        error: isInvalidGrant ? "RefreshAccessTokenError" : "RefreshTemporarilyUnavailable",
        refreshRetryAt: isInvalidGrant ? undefined : Date.now() + REFRESH_RETRY_MS,
      };
    }
    const data = (await response.json()) as {
      access_token?: string;
      refresh_token?: string;
      expires_in?: number;
      refresh_expires_in?: number;
    };
    if (typeof data.access_token !== "string" || !data.access_token) {
      throw new Error("Invalid refresh response");
    }
    if (
      data.refresh_token !== undefined &&
      data.refresh_token !== null &&
      (typeof data.refresh_token !== "string" || !data.refresh_token)
    ) {
      throw new Error("Invalid refresh response");
    }
    const now = Date.now();
    const expiresInSeconds = Number(data.expires_in);
    const refreshExpiresInSeconds = Number(data.refresh_expires_in);
    const nextRefreshToken = data.refresh_token ?? token.refreshToken;
    return {
      ...token,
      accessToken: data.access_token,
      userId: extractUserFromToken(data.access_token).userId,
      refreshToken: nextRefreshToken,
      expiresAt:
        Number.isFinite(expiresInSeconds) && expiresInSeconds > 0
          ? now + expiresInSeconds * 1000
          : now + AUTH_CONFIG.session.accessMaxAge * 1000,
      refreshExpiresAt:
        Number.isFinite(refreshExpiresInSeconds) && refreshExpiresInSeconds > 0
          ? now + refreshExpiresInSeconds * 1000
          : token.refreshExpiresAt,
      error: undefined,
      refreshRetryAt: undefined,
    };
  } catch {
    return {
      ...token,
      error: "RefreshTemporarilyUnavailable",
      refreshRetryAt: Date.now() + REFRESH_RETRY_MS,
    };
  }
}

function tokenFromOidcAccount(
  token: JWT,
  account: {
    access_token: string;
    refresh_token?: string;
    id_token?: string;
    expires_at?: number;
    refresh_expires_in?: number;
  },
): ExtendedJWT {
  const accessToken = account.access_token;
  const now = Date.now();
  const expiresAt =
    typeof account.expires_at === "number"
      ? account.expires_at * 1000
      : now + AUTH_CONFIG.session.accessMaxAge * 1000;
  const refreshExpiresInSeconds = Number(account.refresh_expires_in);
  const refreshExpiresAt =
    Number.isFinite(refreshExpiresInSeconds) && refreshExpiresInSeconds > 0
      ? now + refreshExpiresInSeconds * 1000
      : now + AUTH_CONFIG.session.refreshMaxAge * 1000;
  const decoded = extractUserFromToken(accessToken);
  return {
    ...token,
    accessToken,
    refreshToken: account.refresh_token ?? "",
    idToken: account.id_token,
    expiresAt,
    refreshExpiresAt,
    userId: decoded.userId,
    roles: decoded.roles,
    name: decoded.name ?? token.name,
    email: decoded.email ?? token.email,
    error: undefined,
    refreshRetryAt: undefined,
  } as ExtendedJWT;
}

async function provisionSession(token: ExtendedJWT): Promise<ExtendedJWT> {
  let current = token;
  try {
    const response = await fetch(`${serverApiBaseUrl.replace(/\/$/, "")}/v1/auth/provision`, {
      method: "POST",
      headers: { Authorization: `Bearer ${token.accessToken}` },
      cache: "no-store",
      signal: AbortSignal.timeout(AUTH_UPSTREAM_TIMEOUT_MS),
    });
    if (!response.ok) {
      throw new Error("Account provisioning failed");
    }
    const profile: unknown = await response.json();
    if (
      typeof profile !== "object" ||
      profile === null ||
      !("user_id" in profile) ||
      typeof profile.user_id !== "string"
    ) {
      throw new Error("Invalid application profile");
    }
    const refreshed = await exchangeKeycloakToken(token);
    current = refreshed;
    if (refreshed.error || refreshed.userId !== profile.user_id) {
      throw new Error("Identity link missing from refreshed token");
    }
    return refreshed;
  } catch {
    return {
      ...current,
      error: "AccountProvisioningUnavailable",
      refreshRetryAt: Date.now() + REFRESH_RETRY_MS,
    };
  }
}

export const jwtCallback = async ({
  token,
  account,
}: {
  token: JWT;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  account?: any;
  trigger?: string;
}): Promise<JWT> => {
  if (account?.access_token) {
    return provisionSession(tokenFromOidcAccount(token, account as never));
  }
  if (typeof (token as ExtendedJWT).idToken !== "string" || !(token as ExtendedJWT).idToken) {
    return { ...token, error: "SessionMigrationRequired" } as JWT;
  }
  const current = token as ExtendedJWT;
  if (current.refreshRetryAt && Date.now() < current.refreshRetryAt && current.error) {
    return current;
  }
  if (!current.userId || current.error === "AccountProvisioningUnavailable") {
    if (!current.refreshToken || Date.now() >= current.refreshExpiresAt) {
      return { ...current, error: "RefreshTokenExpired" } as JWT;
    }
    const valid = Date.now() >= current.expiresAt ? await exchangeKeycloakToken(current) : current;
    if (valid.error && valid.error !== "AccountProvisioningUnavailable") {
      return valid;
    }
    return provisionSession(valid);
  }
  if (Date.now() + ACCESS_REFRESH_BUFFER_MS < current.expiresAt) {
    return current;
  }
  if (!current.refreshToken || Date.now() >= current.refreshExpiresAt) {
    return { ...current, error: "RefreshTokenExpired" } as JWT;
  }
  return (await exchangeKeycloakToken(current)) as JWT;
};

function sessionFromToken(session: Session, token: JWT): Session {
  const extToken = token as ExtendedJWT & {
    name?: string;
    email?: string;
  };
  return {
    ...session,
    user: {
      ...session.user,
      id: token.sub ?? "",
      userId: extToken.userId,
      roles: extToken.roles,
      name: extToken.name ?? session.user?.name,
      email: extToken.email ?? session.user?.email,
    },
    error: extToken.error,
  } as Session & { error?: string };
}

export const sessionCallback = async ({
  session,
  token,
}: {
  session: Session;
  token: JWT;
}): Promise<Session> => sessionFromToken(session, token);

export const serverSessionCallback = async ({
  session,
  token,
}: {
  session: Session;
  token: JWT;
}): Promise<Session & { accessToken?: string; accessTokenExpiresAt?: number }> => ({
  ...sessionFromToken(session, token),
  accessToken: (token as ExtendedJWT).accessToken,
  accessTokenExpiresAt: (token as ExtendedJWT).expiresAt,
});

export function accessTokenFromServerSession(
  session: {
    accessToken?: string;
    accessTokenExpiresAt?: number;
    error?: string;
  } | null,
  now = Date.now(),
): string | null {
  if (
    !session?.accessToken ||
    !session.accessTokenExpiresAt ||
    now >= session.accessTokenExpiresAt ||
    (session.error && session.error !== "RefreshTemporarilyUnavailable")
  ) {
    return null;
  }
  return session.accessToken;
}
