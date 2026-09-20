import type { DefaultSession, DefaultUser } from "next-auth";
import type { DefaultJWT } from "next-auth/jwt";

declare module "next-auth" {
  interface Session {
    user: {
      id: string;
      userId?: string;
      roles?: string[];
    } & DefaultSession["user"];
    error?: string;
  }

  interface User extends DefaultUser {
    userId?: string;
    roles?: string[];
    accessToken: string;
    refreshToken: string;
    expiresAt: number;
    refreshExpiresAt: number;
  }
}

declare module "next-auth/jwt" {
  interface JWT extends DefaultJWT {
    userId?: string;
    roles?: string[];
    accessToken: string;
    refreshToken: string;
    expiresAt: number;
    refreshExpiresAt: number;
    idToken?: string;
    error?: string;
    refreshRetryAt?: number;
  }
}
