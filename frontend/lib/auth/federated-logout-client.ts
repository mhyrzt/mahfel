"use client";

export async function getFederatedLogoutUrl(): Promise<string> {
  const callbackUrl = encodeURIComponent("/login?loggedOut=1");
  const response = await fetch(`/api/auth/federated-logout?callbackUrl=${callbackUrl}`, {
    cache: "no-store",
    headers: { Accept: "application/json" },
  });

  if (!response.ok) {
    throw new Error("Federated logout URL is unavailable");
  }

  const payload: unknown = await response.json();
  if (typeof payload !== "object" || payload === null || !("url" in payload)) {
    throw new Error("Federated logout URL is invalid");
  }

  const logoutUrl = payload.url;
  if (typeof logoutUrl !== "string") {
    throw new Error("Federated logout URL is invalid");
  }

  const parsedUrl = new URL(logoutUrl);
  if (parsedUrl.protocol !== "https:" && parsedUrl.protocol !== "http:") {
    throw new Error("Federated logout URL is invalid");
  }

  return parsedUrl.href;
}
