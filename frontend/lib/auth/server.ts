import "server-only";
import { cache } from "react";
import { getServerSession } from "next-auth";
import { serverAuthOptions } from "./options";

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
  const typedSession = session as { accessToken?: string; error?: string } | null;
  if (typedSession?.error === "RefreshTemporarilyUnavailable") {
    return null;
  }
  if (!typedSession?.accessToken || typedSession.error) {
    return null;
  }
  return typedSession.accessToken;
}
