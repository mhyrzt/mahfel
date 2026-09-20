interface JWTPayload {
  sub: string;
  email?: string;
  name?: string;
  preferred_username?: string;
  app_user_id?: string;
  user_id?: string;
  realm_access?: {
    roles?: string[];
  };
}

export function decodeJWT(token: string): JWTPayload {
  if (!token || typeof token !== "string") {
    throw new Error("Invalid token: token must be a non-empty string");
  }
  const parts = token.split(".");
  if (parts.length !== 3) {
    throw new Error("Invalid JWT format: expected 3 parts");
  }
  const payload = parts[1];
  return JSON.parse(Buffer.from(payload, "base64").toString()) as JWTPayload;
}

export function extractUserFromToken(token: string): {
  id: string;
  email: string | undefined;
  name: string | undefined;
  userId: string | undefined;
  roles: string[];
} {
  try {
    const payload = decodeJWT(token);
    return {
      id: payload.sub,
      email: payload.email,
      name: payload.name ?? payload.preferred_username,
      userId: payload.app_user_id ?? payload.user_id,
      roles: payload.realm_access?.roles ?? [],
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to extract user from token: ${message}`);
  }
}
