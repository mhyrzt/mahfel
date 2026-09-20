function parseAge(value: string | undefined, fallback: number): number {
  const parsed = value ? Number(value) : Number.NaN;
  if (Number.isFinite(parsed) && parsed > 0) {
    return parsed;
  }
  return fallback;
}

const sessionMaxAgeSeconds = parseAge(
  process.env.NEXTAUTH_SESSION_MAX_AGE ?? process.env.SESSION_MAX_AGE,
  30 * 24 * 60 * 60,
);

const accessTokenMaxAgeSeconds = parseAge(
  process.env.NEXTAUTH_ACCESS_MAX_AGE ?? process.env.ACCESS_MAX_AGE,
  5 * 60,
);

const refreshTokenMaxAgeSeconds = parseAge(
  process.env.NEXTAUTH_REFRESH_MAX_AGE ?? process.env.REFRESH_MAX_AGE,
  7 * 24 * 60 * 60,
);

const keycloakIssuer =
  process.env.KEYCLOAK_ISSUER?.replace(/\/$/, "") ?? "https://auth.mahfel.localhost/realms/rastai";

export const AUTH_CONFIG = {
  nextAuthUrl: process.env.NEXTAUTH_URL ?? "https://mahfel.localhost",
  nextAuthSecret: process.env.NEXTAUTH_SECRET ?? "",
  keycloak: {
    issuer: keycloakIssuer,
    clientId: process.env.KEYCLOAK_CLIENT_ID ?? "mahfel-web",
    clientSecret: process.env.KEYCLOAK_CLIENT_SECRET ?? "mahfel-web-dev-secret",
    tokenEndpoint: `${keycloakIssuer}/protocol/openid-connect/token`,
  },
  session: {
    maxAge: sessionMaxAgeSeconds,
    accessMaxAge: accessTokenMaxAgeSeconds,
    refreshMaxAge: refreshTokenMaxAgeSeconds,
  },
  pages: {
    signIn: "/login",
    error: "/login",
  },
} as const;
