"use client";

import { signIn } from "next-auth/react";
import { useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";
import { Button } from "@/components/ui/button";
import { PageSkeleton } from "@/features/shell/components/page-skeleton";

function LoginContent() {
  const searchParams = useSearchParams();
  const callbackUrl = searchParams.get("callbackUrl") ?? "/workspace";
  const error = searchParams.get("error");
  const [pending, setPending] = useState(false);

  return (
    <main className="mx-auto flex min-h-screen max-w-md flex-col justify-center gap-6 px-6 py-16">
      <div className="space-y-2">
        <h1 className="text-3xl font-semibold tracking-tight">Sign in to Mahfel</h1>
        <p className="text-sm text-neutral-600">Continue with your organization account.</p>
      </div>
      {error ? (
        <p role="alert" className="rounded-md bg-red-50 px-3 py-2 text-sm text-red-700">
          {error}
        </p>
      ) : null}
      <Button
        disabled={pending}
        onClick={async () => {
          setPending(true);
          await signIn("keycloak", { callbackUrl });
        }}
      >
        {pending ? "Redirecting…" : "Sign in with Keycloak"}
      </Button>
    </main>
  );
}

export default function LoginPage() {
  return (
    <Suspense
      fallback={
        <main className="mx-auto flex min-h-dvh max-w-md flex-col justify-center px-6 py-16">
          <PageSkeleton shape="form" />
        </main>
      }
    >
      <LoginContent />
    </Suspense>
  );
}
