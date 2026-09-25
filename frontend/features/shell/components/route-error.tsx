"use client";

import { useSyncExternalStore } from "react";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { defaultLocale, isLocale, type Locale } from "@/lib/i18n/config";

type Scope = "site" | "workspace" | "rooms" | "sessions";

const copy: Record<
  Locale,
  { title: Record<Scope, string>; detail: string; retry: string; home: string }
> = {
  en: {
    title: {
      site: "This page could not be opened",
      workspace: "The workspace could not be loaded",
      rooms: "Rooms could not be loaded",
      sessions: "Sessions could not be loaded",
    },
    detail: "Something went wrong while loading this page. You can try again.",
    retry: "Try again",
    home: "Go to Mahfel",
  },
  fa: {
    title: {
      site: "باز کردن این صفحه ممکن نشد",
      workspace: "بارگذاری فضای کاری ممکن نشد",
      rooms: "بارگذاری محافل ممکن نشد",
      sessions: "بارگذاری دیدارها ممکن نشد",
    },
    detail: "هنگام بارگذاری این صفحه مشکلی پیش آمد. می‌توانید دوباره تلاش کنید.",
    retry: "تلاش دوباره",
    home: "رفتن به محفل",
  },
};

export function RouteError({
  scope,
  reset,
  locale,
}: Readonly<{ scope: Scope; reset: () => void; locale?: Locale }>) {
  const detectedLocale = useSyncExternalStore(
    () => () => undefined,
    () => {
      const language = document.documentElement.lang;
      return isLocale(language) ? language : defaultLocale;
    },
    () => defaultLocale,
  );
  const text = copy[locale ?? detectedLocale];
  return (
    <section
      aria-labelledby="route-error-title"
      className="mx-auto flex min-h-[20rem] max-w-xl flex-col justify-center gap-5 py-12"
      data-testid={`${scope}-error`}
    >
      <div className="space-y-2">
        <h1 className="text-2xl font-semibold tracking-tight" id="route-error-title">
          {text.title[scope]}
        </h1>
        <p className="text-sm leading-6 text-muted-foreground">{text.detail}</p>
      </div>
      <div className="flex flex-wrap gap-3">
        <Button onClick={reset}>{text.retry}</Button>
        <Button asChild variant="outline">
          <Link href="/">{text.home}</Link>
        </Button>
      </div>
    </section>
  );
}
