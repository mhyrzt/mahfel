import { keycloakLogout } from "@/lib/auth/keycloak";
import type { NextRequest } from "next/server";

export async function GET(request: NextRequest) {
  return keycloakLogout(request);
}
