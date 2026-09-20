import type { NextAuthOptions } from "next-auth";
import { AUTH_CONFIG } from "./config";
import { keycloakProvider } from "./providers";
import { jwtCallback, serverSessionCallback, sessionCallback, signInCallback } from "./callbacks";
import { sessionCookieName, shouldUseSecureAuthCookies } from "./cookie-policy";

const useSecureCookies = shouldUseSecureAuthCookies();

const sharedOptions: Omit<NextAuthOptions, "callbacks"> = {
  providers: [keycloakProvider],
  pages: AUTH_CONFIG.pages,
  session: {
    strategy: "jwt",
    maxAge: AUTH_CONFIG.session.maxAge,
  },
  secret: AUTH_CONFIG.nextAuthSecret,
  cookies: {
    sessionToken: {
      name: sessionCookieName(),
      options: {
        httpOnly: true,
        sameSite: "lax",
        path: "/",
        secure: useSecureCookies,
      },
    },
  },
};

export const authOptions: NextAuthOptions = {
  ...sharedOptions,
  callbacks: {
    signIn: signInCallback,
    jwt: jwtCallback,
    session: sessionCallback,
  },
};

export const serverAuthOptions: NextAuthOptions = {
  ...sharedOptions,
  callbacks: {
    signIn: signInCallback,
    jwt: jwtCallback,
    session: serverSessionCallback,
  },
};
