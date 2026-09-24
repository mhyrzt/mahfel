import "server-only";
import { cache } from "react";

import { createServerApiClient } from "@/lib/api/server";

interface ProfileResponse {
  avatar_url: string | null;
  display_name: string | null;
  email: string | null;
}

export const getProfile = cache(async (): Promise<ProfileResponse | null> => {
  try {
    const api = await createServerApiClient();
    const response = await api.get<ProfileResponse>("/v1/auth/me");
    return response.data;
  } catch {
    return null;
  }
});
