import "server-only";
import { cache } from "react";
import { getServerSession } from "next-auth";
import { serverAuthOptions } from "./options";
import { accessTokenFromServerSession } from "./callbacks";

const getCachedServerSession = cache(() => getServerSession(serverAuthOptions));

export interface ServerSessionUser {
  id?: string;
  userId?: string;
  roles?: string[];
  name?: string | null;
  email?: string | null;
  image?: string | null;
}

export async function getServerSessionUser(): Promise<ServerSessionUser | null> {
  const session = await getCachedServerSession();
  const user = session?.user as ServerSessionUser | undefined | null;
  return user ?? null;
}

export async function getServerAccessToken(): Promise<string | null> {
  const session = await getCachedServerSession();
  return accessTokenFromServerSession(
    session as {
      accessToken?: string;
      accessTokenExpiresAt?: number;
      error?: string;
    } | null,
  );
}
