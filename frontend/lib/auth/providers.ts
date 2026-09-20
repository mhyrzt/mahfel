import KeycloakProvider from "next-auth/providers/keycloak";
import { AUTH_CONFIG } from "./config";

export const keycloakProvider = KeycloakProvider({
  clientId: AUTH_CONFIG.keycloak.clientId,
  clientSecret: AUTH_CONFIG.keycloak.clientSecret,
  issuer: AUTH_CONFIG.keycloak.issuer,
  checks: ["pkce", "state"],
  authorization: { params: { scope: "openid profile email" } },
});
