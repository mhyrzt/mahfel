"use client";

import { signIn } from "next-auth/react";
import { LoaderCircle } from "lucide-react";
import { useState, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import styles from "./landing-page.module.css";

export function KeycloakSignInButton({
  children,
  pendingLabel,
  errorMessage,
}: Readonly<{ children: ReactNode; pendingLabel?: string; errorMessage?: string }>) {
  const [pending, setPending] = useState(false);
  const [failed, setFailed] = useState(false);
  return (
    <div className="space-y-3">
      <Button
        className={`${styles.cta} group h-14 min-w-44 gap-5 rounded-xl px-7 text-sm font-semibold focus-visible:ring-offset-background motion-safe:hover:-translate-y-0.5 active:translate-y-0 motion-reduce:transition-none`}
        disabled={pending}
        aria-busy={pending}
        onClick={async () => {
          setPending(true);
          setFailed(false);
          try {
            await signIn("keycloak", { callbackUrl: "/workspace" });
          } catch {
            setFailed(true);
          } finally {
            setPending(false);
          }
        }}
      >
        {pending && <LoaderCircle aria-hidden="true" className="size-4 motion-safe:animate-spin" />}
        {pending && pendingLabel ? pendingLabel : children}
      </Button>
      {failed && errorMessage && (
        <p role="alert" className="max-w-sm text-sm leading-6 text-destructive">
          {errorMessage}
        </p>
      )}
    </div>
  );
}
