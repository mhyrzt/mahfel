"use client";

import { signIn } from "next-auth/react";
import { useState, type ReactNode } from "react";
import { Button } from "@/components/ui/button";

export function KeycloakSignInButton({ children }: Readonly<{ children: ReactNode }>) {
  const [pending, setPending] = useState(false);
  return (
    <Button
      className="h-12 gap-3 rounded-xl px-5 text-sm font-semibold"
      disabled={pending}
      onClick={async () => {
        setPending(true);
        try {
          await signIn("keycloak", { callbackUrl: "/workspace" });
        } finally {
          setPending(false);
        }
      }}
    >
      {children}
    </Button>
  );
}
